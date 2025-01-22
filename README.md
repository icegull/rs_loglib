# rs_loglib

A high-performance, thread-safe logging library for Rust with rolling file support.

## Quick Start

```rust
use rs_loglib::{LogConfig, info};

fn main() {
    // Initialize a logger instance
    let instance = rs_loglib::init_logger(
        LogConfig::new()
            .with_instance_name("myapp")
            .with_file_name("app.log")
    ).unwrap();

    // Log messages
    info!(instance, "Hello from rs_loglib!");
}
```

## Configuration Options

The following table lists all available configuration options and their default values:

| Option | Type | Default Value | Description |
|--------|------|---------------|-------------|
| `log_path` | `PathBuf` | `"C:/logs/"` | Base directory for log files |
| `max_files` | `u32` | `5` | Maximum number of backup files to keep |
| `max_size` | `u64` | `20 * 1024 * 1024` | Maximum size per log file in bytes (20MB) |
| `is_async` | `bool` | `true` | Enable asynchronous logging |
| `instant_flush` | `bool` | `false` | Enable automatic buffer flushing |
| `file_name` | `String` | `"record"` | Base name for log files |
| `instance_name` | `String` | `"default"` | Unique identifier for logger instance |

### File Naming Convention

- Main log file: `{file_name}.log`
- Rotated files: `{file_name}.1.log`, `{file_name}.2.log`, etc.
- Example: `app.log`, `app.1.log`, `app.2.log`

### Configuration Example

```rust
let config = LogConfig::new()
    .with_path("/var/log")               // Override default log path
    .with_file_name("myapp.log")         // Custom file name
    .with_max_size(10 * 1024 * 1024)     // 10MB per file
    .with_max_files(3)                   // Keep 3 backup files
    .with_async(true)                    // Use async logging
    .with_auto_flush(true)              // Enable auto flush
    .with_instance_name("myapp")       // Custom instance name
    .with_instant_flush("true");        //Enable instant flush
```

### Log Level Reference

Available log levels in order of severity:

| Level | Method | Usage |
|-------|--------|-------|
| DEBUG | `debug!()` | Detailed information for debugging |
| INFO | `info!()` | General operational messages |
| WARN | `warn!()` | Warning messages for potential issues |
| ERROR | `error!()` | Error conditions |
| FATAL | `fatal!()` | Critical errors (terminates program) |

## Key Features

- **Multiple Logger Instances**: Run multiple loggers with different configurations
- **Rolling File Support**: Automatically rotate logs based on file size
- **Thread Safety**: Safe for concurrent use across multiple threads
- **Asynchronous Logging**: Optional non-blocking logging operations
- **Structured Output**: Timestamp, log level, and thread ID in each log entry

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rs_loglib = { git = "https://github.com/icegull/rs_loglib" }
```

## Core Concepts

### Logger Configuration

The `LogConfig` builder provides a fluent interface for configuration:

```rust
let config = LogConfig::new()
    .with_path("/var/log")          // Base log directory
    .with_file_name("app.log")      // Log file name
    .with_max_size(10 * 1024 * 1024)// Max file size (10MB)
    .with_max_files(5)              // Keep 5 backup files
    .with_async(true)               // Enable async logging
    .with_instance_name("app1");    // Unique instance name
```

### Log Levels

Five log levels are available:

```rust
debug!(instance, "Debug information");
info!(instance, "Normal operation");
warn!(instance, "Warning condition");
error!(instance, "Error condition");
fatal!(instance, "Fatal error");  // Will terminate the program
```

### Multiple Loggers

Each logger instance is independent and can have its own configuration:

```rust
// Application logger
let app_logger = init_logger(LogConfig::new()
    .with_instance_name("app")
    .with_file_name("app.log")
).unwrap();

// Access logger
let access_logger = init_logger(LogConfig::new()
    .with_instance_name("access")
    .with_file_name("access.log")
    .with_async(true)
).unwrap();

// Use different loggers
info!(app_logger, "Application event");
info!(access_logger, "Access event");
```

### Log Format

Each log entry follows this format:
```
TIMESTAMP [LEVEL][THREAD] MESSAGE
```

Example output:
```
2024-01-20 15:30:45.123 [info][1234] Server started on port 8080
2024-01-20 15:30:45.125 [error][1234] Failed to connect to database
```

## Performance Considerations

- Enable async logging for high-throughput scenarios
- Use appropriate max_size to balance between file size and rotation frequency
- Consider file system performance when setting log directory location

## Thread Safety

All logging operations are thread-safe. The library uses:
- Atomic operations for counters
- Mutex protection for file operations
- Lock-free algorithms where possible

## Error Handling

The library provides Result types for initialization:

```rust
match init_logger(config) {
    Ok(instance) => info!(instance, "Logger initialized"),
    Err(e) => panic!("Failed to initialize logger: {}", e),
}
```

## Memory Usage

Log rotation helps manage disk space:
- Old files are automatically removed
- File size is monitored and managed
- Async logging buffers are sized appropriately

## License

MIT License. See [LICENSE](LICENSE) for details.
