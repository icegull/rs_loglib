# rs_loglib

A thread-safe Rust logging library with file rotation and async support, built on top of `tracing`.

## Features

- Size-based log rotation with configurable file size
- Backup file management with configurable file count
- Async and sync logging support
- Thread-safe implementation with efficient locking
- Local timezone support
- Customizable log file names and paths
- Simple builder-style configuration

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rs_loglib = { path = "path/to/rs_loglib" }
```

## Quick Start

```rust
use rs_loglib::{LogConfig, info, error, warn, debug};

fn main() {
    // Basic configuration
    let config = LogConfig::new()
        .with_path("C:/logs")              // Log directory
        .with_file_name("myapp.log")        // Base name for log files
        .with_max_size(10 * 1024 * 1024) // 10MB per file
        .with_max_files(5)              // Keep 5 backup files
        .with_async(true);              // Use async logging

    rs_loglib::init(config);

    // Use logging macros
    info!("Application started");
    error!("Error: {}", "connection failed");
    debug!("Debug info: {}", "detail");
}
```

## Configuration Options

| Option | Method | Default | Description |
|--------|---------|---------|-------------|
| Log directory | `with_path()` | "C:/logs/" | Base directory for log files |
| File name | `with_file_name()` | "record.log" | Base name for log files |
| Max size | `with_max_size()` | 20MB | Size threshold for rotation |
| Max files | `with_max_files()` | 5 | Number of backup files to keep |
| Async mode | `with_async()` | true | Enable async logging |
| Auto flush | `with_auto_flush()` | false | Enable automatic flushing |

## Log File Management

The library manages log files using the following pattern:
- Active log: `{file_name}.log`
- Backups: `{file_name}.1.log`, `{file_name}.2.log`, etc.

When a log file reaches the configured size:
1. Current file is renamed to `.1.log`
2. Previous backups are shifted up
3. Oldest backup is deleted if exceeding max_files
4. New log file is created

## Log Format

Logs are written in the following format:
```
2024-01-21 11:58:36.150 [error][7257] Error message here
```

Components:
- Timestamp in local timezone
- Log level in lowercase
- Thread ID (hashed to 4 digits)
- Message content

## Thread Safety

The library provides thread-safe logging through:
- `parking_lot::Mutex` for efficient locking
- Optional async logging via `tracing-appender`
- Thread-safe file rotation mechanism

## Examples

### Basic Synchronous Logging
```rust
let config = LogConfig::new()
    .with_path("C:/logs")
    .with_file_name("sync_app.log")
    .with_async(false);

rs_loglib::init(config);
info!("Sync logging initialized");
```

### Async Logging with Custom Size
```rust
let config = LogConfig::new()
    .with_path("C:/logs/")
    .with_file_name("async_app.log")
    .with_max_size(5 * 1024 * 1024)  // 5MB
    .with_max_files(3)
    .with_async(true);

rs_loglib::init(config);
info!("Async logging initialized");
```

## License

Licensed under either of:
- Apache License, Version 2.0
- MIT license

at your option.
