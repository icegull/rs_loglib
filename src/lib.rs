#[macro_use]
extern crate lazy_static;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::io;
pub use std::io::Write;
use std::fs::{File, OpenOptions};
use parking_lot::Mutex;
use std::fs;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use tracing::Level;
use time::macros::format_description;

pub(crate) struct FileState {
    file: File,
    size: u64,
}

pub struct RollingFileWriter {
    state: Mutex<FileState>,
    base_path: PathBuf,
    max_size: u64,
    max_files: u32,
    instant_flush: bool, 
}

impl RollingFileWriter {
    fn new(base_path: PathBuf, max_size: u64, max_files: u32, instant_flush: bool) -> io::Result<Self> {
        let path = base_path.with_extension("log");
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        let size = file.metadata()?.len();

        Ok(Self {
            state: Mutex::new(FileState { file, size }),
            base_path,
            max_size,
            max_files,
            instant_flush,
        })
    }

    fn rotate(&self) -> io::Result<()> {
        let state = self.state.lock();
        let current_file = &state.file;
        
        // Close current file
        current_file.sync_all()?;
        drop(state);  // Release lock early

        // Rotate existing files
        for i in (1..self.max_files).rev() {
            let src = self.base_path.with_extension(format!("{}.log", i));
            let dst = self.base_path.with_extension(format!("{}.log", i + 1));
            if src.exists() {
                fs::rename(&src, &dst)?;
            }
        }

        // Move current file to .1
        let current = self.base_path.with_extension("log");
        let backup = self.base_path.with_extension("1.log");
        fs::rename(&current, &backup)?;

        // Create new file
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&current)?;
        
        let mut state = self.state.lock();
        state.file = file;
        state.size = 0;

        // Remove oldest file if exists
        let oldest = self.base_path.with_extension(format!("{}.log", self.max_files));
        if oldest.exists() {
            fs::remove_file(oldest)?;
        }

        Ok(())
    }
}

impl Write for RollingFileWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut state = self.state.lock();
        if state.size + buf.len() as u64 > self.max_size {
            drop(state);
            self.rotate()?;
            let mut state = self.state.lock();
            let written = state.file.write(buf)?;
            state.size += written as u64;
            if self.instant_flush {
                state.file.flush()?;
            }
            Ok(written)
        } else {
            let written = state.file.write(buf)?;
            state.size += written as u64;
            if self.instant_flush {
                state.file.flush()?;
            }
            Ok(written)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.state.lock().file.flush()
    }
}

impl Write for &RollingFileWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut state = self.state.lock();
        if state.size + buf.len() as u64 > self.max_size {
            drop(state);
            self.rotate()?;
            let mut state = self.state.lock();
            let written = state.file.write(buf)?;
            state.size += written as u64;
            if self.instant_flush {
                state.file.flush()?;
            }
            Ok(written)
        } else {
            let written = state.file.write(buf)?;
            state.size += written as u64;
            if self.instant_flush {
                state.file.flush()?;
            }
            Ok(written)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.state.lock().file.flush()
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

impl Write for &WriterWrapper {
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
            log_path: PathBuf::from("C:/logs/"),
            max_files: 5,
            max_size: 20 * 1024 * 1024,
            is_async: true,
            instant_flush: false,
            file_name: String::from("record"),
            instance_name: String::from("default"),
        }
    }
}

impl LogConfig {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.log_path = path.as_ref().to_path_buf();
        self
    }

    pub fn with_max_files(mut self, count: u32) -> Self {
        self.max_files = count;
        self
    }

    pub fn with_max_size(mut self, size: u64) -> Self {
        self.max_size = size;
        self
    }

    pub fn with_async(mut self, is_async: bool) -> Self {
        self.is_async = is_async;
        self
    }

    pub fn with_instant_flush(mut self, instant_flush: bool) -> Self {
        self.instant_flush = instant_flush;
        self
    }

    pub fn with_file_name<S: Into<String>>(mut self, name: S) -> Self {
        self.file_name = name.into();
        self
    }

    pub fn with_instance_name(mut self, name: &str) -> Self {
        self.instance_name = name.to_string();
        self
    }
}

pub struct Logger {
    writer: WriterWrapper,
    _guard: Option<tracing_appender::non_blocking::WorkerGuard>,
}

impl Logger {
    pub fn log(&self, level: Level, message: &str) -> io::Result<()> {
        let now = time::OffsetDateTime::now_local().unwrap_or(time::OffsetDateTime::now_utc());
        let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]");
        let timestamp = now.format(&format).unwrap_or_default();
        
        let thread_id = std::thread::current().id();
        let mut hasher = DefaultHasher::new();
        thread_id.hash(&mut hasher);
        let thread_hash = hasher.finish() % 10000;

        let log_line = format!(
            "{} [{}][{}] {}\n",
            timestamp,
            level.as_str().to_lowercase(),
            thread_hash,
            message
        );

        let mut writer = self.writer.clone();
        writer.write_all(log_line.as_bytes())
    }
}

lazy_static! {
    pub static ref LOGGER_INSTANCES: Mutex<HashMap<String, Logger>> = Mutex::new(HashMap::new());
}

pub fn init_logger(config: LogConfig) -> Result<String, io::Error> {
    let instance_name = config.instance_name.clone();
    let process_name = std::env::current_exe()
        .ok()
        .and_then(|pb| pb.file_name().map(|s| s.to_string_lossy().into_owned()))
        .unwrap_or_else(|| String::from("unknown"));

    let log_dir = config.log_path.join(&process_name);
    std::fs::create_dir_all(&log_dir).expect("Failed to create log directory");

    let log_path = log_dir.join(&config.file_name); 
    let file_writer = WriterWrapper(Arc::new(RollingFileWriter::new(
        log_path,
        config.max_size,
        config.max_files,
        config.instant_flush,  // Pass instant_flush setting
    ).expect("Failed to create rolling file writer")));

    let guard = if config.is_async {
        let (_writer, guard) = tracing_appender::non_blocking(file_writer.clone());
        Some(guard)
    } else {
        None
    };

    let logger = Logger {
        writer: file_writer,
        _guard: guard,
    };

    LOGGER_INSTANCES.lock().insert(instance_name.clone(), logger);
    Ok(instance_name)
}

#[macro_export]
macro_rules! info {
    ($instance:expr, $($arg:tt)*) => {{
        if let Some(logger) = $crate::LOGGER_INSTANCES.lock().get(&$instance) {
            let _ = logger.log(tracing::Level::INFO, &format!($($arg)*));
        }
    }};
}

#[macro_export]
macro_rules! error {
    ($instance:expr, $($arg:tt)*) => {{
        if let Some(logger) = $crate::LOGGER_INSTANCES.lock().get(&$instance) {
            let _ = logger.log(tracing::Level::ERROR, &format!($($arg)*));
        }
    }};
}

#[macro_export]
macro_rules! warn {
    ($instance:expr, $($arg:tt)*) => {{
        if let Some(logger) = $crate::LOGGER_INSTANCES.lock().get(&$instance) {
            let _ = logger.log(tracing::Level::WARN, &format!($($arg)*));
        }
    }};
}

#[macro_export]
macro_rules! debug {
    ($instance:expr, $($arg:tt)*) => {{
        if let Some(logger) = $crate::LOGGER_INSTANCES.lock().get(&$instance) {
            let _ = logger.log(tracing::Level::DEBUG, &format!($($arg)*));
        }
    }};
}

#[macro_export]
macro_rules! fatal {
    ($instance:expr, $($arg:tt)*) => {{
        if let Some(logger) = $crate::LOGGER_INSTANCES.lock().get(&$instance) {
            let _ = logger.log(tracing::Level::ERROR, &format!("FATAL: {}", format!($($arg)*)));
        }
        std::process::exit(1);
    }};
}
