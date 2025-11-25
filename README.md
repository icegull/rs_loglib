# rs_loglib

A high-performance, thread-safe logging library for Rust with rolling file support.

## Features

- **Rolling File Support**: Automatically rotate logs based on file size
- **Multiple Logger Instances**: Run multiple independent loggers with different configurations
- **Thread Safety**: Safe for concurrent use across multiple threads using `parking_lot` mutex
- **Instant Flush Option**: Optional immediate buffer flushing for critical logging scenarios
- **Structured Output**: Timestamp, thread ID, and log level in each log entry

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rs_loglib = { git = "https://github.com/icegull/rs_loglib" }
```

## Quick Start

```rust
use rs_loglib::{LogConfig, init_logger, info};

fn main() {
    // Initialize a logger instance
    let logger = init_logger(
        LogConfig::new()
            .with_instance_name("myapp")
            .with_file_name("app.log")
    ).unwrap();

    // Log messages
    info!(logger, "Hello from rs_loglib!");
}
```

## Configuration Options

The following table lists all available configuration options and their default values:

| Option | Type | Default Value | Description |
|--------|------|---------------|-------------|
| `log_path` | `PathBuf` | `"logs"` | Base directory for log files |
| `max_files` | `u32` | `5` | Maximum number of backup files to keep |
| `max_size` | `u64` | `20 * 1024 * 1024` (20MB) | Maximum size per log file in bytes |
| `is_async` | `bool` | `false` | Reserved for async logging (not yet implemented) |
| `instant_flush` | `bool` | `false` | Enable instant buffer flushing |
| `file_name` | `String` | `"app"` | Base name for log files |
| `instance_name` | `String` | `"default"` | Unique identifier for logger instance |

### Configuration Example

```rust
use rs_loglib::LogConfig;

let config = LogConfig::new()
    .with_path("/var/log/myapp")         // Custom log directory
    .with_file_name("server.log")        // Custom file name
    .with_max_size(10 * 1024 * 1024)     // 10MB per file
    .with_max_files(3)                   // Keep 3 backup files
    .with_instant_flush(true)            // Enable instant flush
    .with_instance_name("server");       // Instance identifier
```

### File Naming Convention

- Main log file: `{file_name}.log`
- Rotated files: `{file_name}.1.log`, `{file_name}.2.log`, etc.
- Example: `server.log` → `server.1.log` → `server.2.log`

## Log Levels

Four log levels are available in order of severity:

| Level | Macro | Usage |
|-------|-------|-------|
| ERROR | `error!()` | Error conditions |
| WARN | `warn!()` | Warning messages for potential issues |
| INFO | `info!()` | General operational messages |
| DEBUG | `debug!()` | Detailed information for debugging |

Additionally, there is a `fatal!()` macro that logs an ERROR level message with "FATAL:" prefix and terminates the program.

```rust
use rs_loglib::{debug, info, warn, error, fatal};

debug!(logger, "Debug information: {:?}", data);
info!(logger, "Server started on port {}", port);
warn!(logger, "Connection timeout after {} seconds", timeout);
error!(logger, "Failed to connect: {}", err);
fatal!(logger, "Unrecoverable error");  // Terminates the program
```

## Log Format

Each log entry follows this format:

```
TIMESTAMP [THREAD_ID][LEVEL] MESSAGE
```

Example output:

```
2024-01-20 15:30:45.123 [01234][INFO ] Server started on port 8080
2024-01-20 15:30:45.125 [01234][ERROR] Failed to connect to database
```

- **TIMESTAMP**: Local time in `YYYY-MM-DD HH:MM:SS.mmm` format
- **THREAD_ID**: 5-digit hash of the thread ID
- **LEVEL**: Log level (left-aligned, 5 characters)

## Multiple Loggers

Each logger instance is independent and can have its own configuration:

```rust
use rs_loglib::{LogConfig, init_logger, info, error, warn};

// Application logger
let app_logger = init_logger(LogConfig::new()
    .with_instance_name("app")
    .with_path("logs")
    .with_file_name("app.log")
).unwrap();

// Access logger with different settings
let access_logger = init_logger(LogConfig::new()
    .with_instance_name("access")
    .with_path("logs")
    .with_file_name("access.log")
    .with_max_files(10)
).unwrap();

// Use different loggers
info!(app_logger, "Application started");
info!(access_logger, "GET /api/users 200");
```

## Thread Safety

All logging operations are thread-safe. The library uses `parking_lot::Mutex` for efficient synchronization:

```rust
use rs_loglib::{LogConfig, init_logger, info};
use std::thread;

let logger = init_logger(LogConfig::new()
    .with_instance_name("threaded")
    .with_file_name("threaded.log")
).unwrap();

let logger_clone = logger.clone();

let handle = thread::spawn(move || {
    for i in 0..1000 {
        info!(logger_clone, "Message {} from spawned thread", i);
    }
});

for i in 0..1000 {
    info!(logger, "Message {} from main thread", i);
}

handle.join().unwrap();
```

## Error Handling

The `init_logger` function returns a `Result` type for proper error handling:

```rust
use rs_loglib::{LogConfig, init_logger, info};

match init_logger(LogConfig::new().with_instance_name("app")) {
    Ok(logger) => {
        info!(logger, "Logger initialized successfully");
    }
    Err(e) => {
        eprintln!("Failed to initialize logger: {}", e);
    }
}
```

## Log Rotation

Log rotation helps manage disk space:

- When a log file exceeds `max_size`, it is automatically rotated
- Existing backup files are shifted (`app.1.log` → `app.2.log`, etc.)
- Files exceeding `max_files` count are automatically removed
- A new empty log file is created for continued logging

## Dependencies

- `parking_lot`: High-performance synchronization primitives
- `time`: Date and time formatting
- `lazy_static`: Lazy static initialization

## License

MIT License. See [LICENSE](LICENSE) for details.
