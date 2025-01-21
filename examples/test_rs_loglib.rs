use core::error;

use rs_loglib::{debug, error, fatal, info, warn, LogConfig};

fn main() {
    // Initialize the logger with custom configuration
    let config = LogConfig::new()
        .with_path("C:/logs")
        .with_file_name("test.log")
        .with_max_files(5)
        .with_max_size(20 * 1024 * 1024)
        .with_async(true)
        .with_auto_flush(false);

    rs_loglib::init(config);

    info!("This is an info message");
    error!("This is an error message");
    warn!("This is a warning message");
    debug!("This is a debug message");
    info!("Hello, {}!", "world");
    error!("This is my number {}", 1);

    // fatal!("This is a fatal message"); // This will terminate the program
}
