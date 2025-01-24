use rs_loglib::{error, info, warn, LogConfig, Level};
use std::thread;

fn main() {
    let config1 = LogConfig::new()
        .with_instance_name("app1")
        .with_path("C:/logs/")
        .with_file_name("app1.log")
        .with_instant_flush(false)
        .with_max_files(5);

    let logger1 = rs_loglib::init_logger(config1).unwrap();

    let config2 = LogConfig::new()
        .with_instance_name("app2")
        .with_path("C:/logs/")
        .with_file_name("app2.log")
        .with_max_files(3);

    let logger2 = rs_loglib::init_logger(config2).unwrap();

    let logger1_clone = logger1.clone();
    let logger2_clone = logger2.clone();

    let thread1 = thread::spawn(move || {
        for i in 0..1000 {
            info!(logger1_clone, "Message {} from app1", i);
            error!(logger1_clone, "Error {} in app1", i);
        }
    });

    let thread2 = thread::spawn(move || {
        for i in 0..1000 {
            info!(logger2_clone, "Message {} from app2", i);
            warn!(logger2_clone, "Warning {} in app2", i);
        }
    });

    thread1.join().unwrap();
    thread2.join().unwrap();
    println!("Threads completed");
}
