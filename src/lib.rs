extern crate lazy_static;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::io::{self, Write};
use std::fs::{self, File, OpenOptions};
use parking_lot::Mutex;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use time::macros::format_description;

#[derive(Debug, Clone, Copy)]
pub enum Level {
    ERROR,
    WARN,
    INFO,
    DEBUG,
}

impl Level {
    fn as_str(&self) -> &'static str {
        match self {
            Level::ERROR => "ERROR",
            Level::WARN => "WARN",
            Level::INFO => "INFO",
            Level::DEBUG => "DEBUG",
        }
    }
}

struct InnerState {
    file: File,
    current_size: u64,
}

pub struct RollingFileWriter {
    state: Mutex<InnerState>,
    base_path: PathBuf,
    max_size: u64,
    max_files: u32,
    instant_flush: bool, 
}

impl RollingFileWriter {
    fn new(base_path: PathBuf, max_size: u64, max_files: u32, instant_flush: bool) -> io::Result<Self> {
        if let Some(parent) = base_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let path = base_path.with_extension("log");
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        let size = file.metadata()?.len();

        Ok(Self {
            state: Mutex::new(InnerState { file, current_size: size }),
            base_path,
            max_size,
            max_files,
            instant_flush,
        })
    }

    fn rotate_locked(&self, state: &mut InnerState) -> io::Result<()> {
        if state.current_size < self.max_size {
            return Ok(());
        }

        state.file.sync_all()?;
        
        let get_path = |idx: u32| -> PathBuf {
            if idx == 0 {
                self.base_path.with_extension("log")
            } else {
                self.base_path.with_extension(format!("{}.log", idx))
            }
        };

        for i in (0..self.max_files - 1).rev() {
            let src = get_path(i);
            let dst = get_path(i + 1);
            
            if src.exists() {
                if dst.exists() {
                    let _ = fs::remove_file(&dst); 
                }
                let _ = fs::rename(&src, &dst);
            }
        }

        let path = self.base_path.with_extension("log");
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        
        state.file = file;
        state.current_size = 0;
        
        Ok(())
    }
}

impl Write for &RollingFileWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut state = self.state.lock();
        
        if state.current_size + buf.len() as u64 > self.max_size {
            if let Err(e) = self.rotate_locked(&mut state) {
                eprintln!("Log rotation failed: {}", e);
            }
        }

        let written = state.file.write(buf)?;
        state.current_size += written as u64;
        
        if self.instant_flush {
            state.file.flush()?;
        }
        
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut state = self.state.lock();
        state.file.flush()
    }
}

#[derive(Clone)]
pub struct WriterWrapper(pub(crate) Arc<RollingFileWriter>);

impl Write for WriterWrapper {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (&*self.0).write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        (&*self.0).flush()
    }
}

#[derive(Debug)]
pub struct LogConfig {
    log_path: PathBuf,
    max_files: u32,
    max_size: u64,
    is_async: bool,
    instant_flush: bool,
    file_name: String,
    instance_name: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            log_path: PathBuf::from("logs"),
            max_files: 5,
            max_size: 20 * 1024 * 1024,
            is_async: false,
            instant_flush: false,
            file_name: String::from("app"),
            instance_name: String::from("default"),
        }
    }
}

impl LogConfig {
    pub fn new() -> Self { Default::default() }
    pub fn with_path<P: AsRef<Path>>(mut self, path: P) -> Self { self.log_path = path.as_ref().to_path_buf(); self }
    pub fn with_max_files(mut self, count: u32) -> Self { self.max_files = count; self }
    pub fn with_max_size(mut self, size: u64) -> Self { self.max_size = size; self }
    pub fn with_async(mut self, is_async: bool) -> Self { self.is_async = is_async; self }
    pub fn with_instant_flush(mut self, instant_flush: bool) -> Self { self.instant_flush = instant_flush; self }
    pub fn with_file_name<S: Into<String>>(mut self, name: S) -> Self { self.file_name = name.into(); self }
    pub fn with_instance_name(mut self, name: &str) -> Self { self.instance_name = name.to_string(); self }
}

#[derive(Clone)]
pub struct Logger {
    writer: WriterWrapper,
    #[allow(dead_code)]
    instance_name: String,
}

impl Logger {
    pub fn log(&self, level: Level, message: &str) -> io::Result<()> {
        thread_local! {
            static THREAD_ID_STR: String = {
                let thread_id = std::thread::current().id();
                let mut hasher = DefaultHasher::new();
                thread_id.hash(&mut hasher);
                let thread_hash = hasher.finish() % 10000;
                format!("{:05}", thread_hash)
            };
        }

        let log_line = THREAD_ID_STR.with(|tid_str| {
            format_log_message(level.as_str(), tid_str, message)
        });
        
        let mut writer = self.writer.clone();
        writer.write_all(log_line.as_bytes())
    }
}

fn format_log_message(level: &str, thread_id_str: &str, message: &str) -> String {
    let now = time::OffsetDateTime::now_local().unwrap_or(time::OffsetDateTime::now_utc());
    let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]");
    let timestamp = now.format(&format).unwrap_or_default();
    
    format!(
        "{} [{}][{:<5}] {}\n",
        timestamp,
        thread_id_str,
        level,
        message
    )
}

pub fn init_logger(config: LogConfig) -> Result<Logger, io::Error> {
    let instance_name = config.instance_name.clone();
    
    let log_dir = &config.log_path;
    std::fs::create_dir_all(log_dir)?;

    let file_stem = Path::new(&config.file_name).file_stem().unwrap_or(std::ffi::OsStr::new("app"));
    let log_path = log_dir.join(file_stem); 

    if config.is_async {
        eprintln!("Warning: Async logging requested but not implemented. Falling back to sync.");
    }

    let file_writer = WriterWrapper(Arc::new(RollingFileWriter::new(
        log_path,
        config.max_size,
        config.max_files,
        config.instant_flush,
    )?));

    Ok(Logger {
        writer: file_writer,
        instance_name,
    })
}

#[macro_export]
macro_rules! info {
    ($logger:expr, $($arg:tt)*) => {{
        let _ = $logger.log($crate::Level::INFO, &format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! error {
    ($logger:expr, $($arg:tt)*) => {{
        let _ = $logger.log($crate::Level::ERROR, &format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! warn {
    ($logger:expr, $($arg:tt)*) => {{
        let _ = $logger.log($crate::Level::WARN, &format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! debug {
    ($logger:expr, $($arg:tt)*) => {{
        let _ = $logger.log($crate::Level::DEBUG, &format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! fatal {
    ($logger:expr, $($arg:tt)*) => {{
        let _ = $logger.log($crate::Level::ERROR, &format!("FATAL: {}", format!($($arg)*)));
        std::process::exit(1);
    }};
}
