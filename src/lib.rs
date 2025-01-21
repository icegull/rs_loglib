use std::path::{Path, PathBuf};
use std::sync::{Once, Arc};
use std::io::{self, Write};
use std::fs::{File, OpenOptions};
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::registry::LookupSpan;
use parking_lot::Mutex;
use std::fs;
use time::macros::format_description;
use tracing_subscriber::fmt::time::LocalTime;
use tracing_subscriber::fmt::format::{Writer, FormatEvent, FormatFields};
use tracing::Subscriber;
use tracing::Event;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

static INIT: Once = Once::new();

struct FileState {
    file: File,
    size: u64,
}

struct RollingFileWriter {
    state: Mutex<FileState>,
    base_path: PathBuf,
    max_size: u64,
    max_files: u32,
}

impl RollingFileWriter {
    fn new(base_path: PathBuf, max_size: u64, max_files: u32) -> io::Result<Self> {
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
            Ok(written)
        } else {
            let written = state.file.write(buf)?;
            state.size += written as u64;
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
            Ok(written)
        } else {
            let written = state.file.write(buf)?;
            state.size += written as u64;
            Ok(written)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.state.lock().file.flush()
    }
}

#[derive(Clone)]
struct WriterWrapper(Arc<RollingFileWriter>);

impl Write for WriterWrapper {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (&*self.0).write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        (&*self.0).flush()
    }
}

impl<'a> MakeWriter<'a> for WriterWrapper {
    type Writer = Self;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

#[derive(Debug)]
pub struct LogConfig {
    log_path: PathBuf,
    max_files: u32,
    max_size: u64,
    is_async: bool,
    auto_flush: bool,
    file_name: String,  // 新增字段
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            log_path: PathBuf::from("C:/logs/"),
            max_files: 5,
            max_size: 20 * 1024 * 1024,
            is_async: true,
            auto_flush: false,
            file_name: String::from("record"),  // 默认文件名
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

    pub fn with_auto_flush(mut self, auto_flush: bool) -> Self {
        self.auto_flush = auto_flush;
        self
    }

    pub fn with_file_name<S: Into<String>>(mut self, name: S) -> Self {
        self.file_name = name.into();
        self
    }
}

struct CustomFormatter<T> {
    timer: T,
}

impl<S, N, T> FormatEvent<S, N> for CustomFormatter<T>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
    T: tracing_subscriber::fmt::time::FormatTime,
{
    fn format_event(
        &self,
        ctx: &tracing_subscriber::fmt::FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        // Write timestamp
        self.timer.format_time(&mut writer)?;
        writer.write_char(' ')?;

        // Write level
        let level = event.metadata().level();
        write!(writer, "[{}]", level.as_str().to_lowercase())?;

        // Write thread id using hash of thread id
        let thread_id = std::thread::current().id();
        let mut hasher = DefaultHasher::new();
        thread_id.hash(&mut hasher);
        write!(writer, "[{}]", hasher.finish() % 10000)?; // Use modulo to keep it readable

        // Write message
        writer.write_char(' ')?;
        ctx.format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}

pub fn init(config: LogConfig) {
    INIT.call_once(|| {
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
        ).expect("Failed to create rolling file writer")));

        let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]");
        let timer = LocalTime::new(format);
        let formatter = CustomFormatter { timer: timer.clone() };

        let subscriber_builder = tracing_subscriber::fmt()
            .with_max_level(LevelFilter::TRACE)
            .with_ansi(false)
            .with_writer(file_writer.clone())
            .event_format(formatter);

        if config.is_async {
            use std::sync::Mutex;
            static GUARD: Mutex<Option<tracing_appender::non_blocking::WorkerGuard>> = Mutex::new(None);
            let (non_blocking, guard) = tracing_appender::non_blocking(file_writer);
            *GUARD.lock().unwrap() = Some(guard);
            subscriber_builder.with_writer(non_blocking).init();
        } else {
            subscriber_builder.init();
        }
    });
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        tracing::info!($($arg)*);
    }
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        tracing::error!($($arg)*);
    }
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        tracing::warn!($($arg)*);
    }
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        tracing::debug!($($arg)*);
    }
}

#[macro_export]
macro_rules! fatal {
    ($($arg:tt)*) => {
        tracing::error!("FATAL: {}", format!($($arg)*));
        std::process::exit(1);
    }
}
