use rs_loglib::{error, info, warn, LogConfig};
use std::thread;

fn main() {
    let config1 = LogConfig::new()
        .with_instance_name("app1")
        .with_path("C:/logs/")
        .with_file_name("app1.log")
        .with_instant_flush(false)
        .with_max_files(5);

    let instance1 = rs_loglib::init_logger(config1).unwrap();

    let config2 = LogConfig::new()
        .with_instance_name("app2")
        .with_path("C:/logs/")
        .with_file_name("app2.log")
        .with_max_files(3);

    let instance2 = rs_loglib::init_logger(config2).unwrap();

    // Clone the instances for thread use
    let instance1_clone = instance1.clone();
    let instance2_clone = instance2.clone();

    // Spawn thread for app1 logging
    let thread1 = thread::spawn(move || {
        for i in 0..10000 {
            info!(instance1_clone, "Message {} from app1", i);
            error!(instance1_clone, "Error {} in app1", i);
           // thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    // Spawn thread for app2 logging
    let thread2 = thread::spawn(move || {
        for i in 0..10000 {
            info!(instance2_clone, "Message {} from app2", i);
            warn!(instance2_clone, "Warning {} in app2", i);
           // thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    // Wait for both threads to complete
    thread1.join().unwrap();
    thread2.join().unwrap();
    println!("Threads completed");
}
