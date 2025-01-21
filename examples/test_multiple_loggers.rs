use rs_loglib::{error, info, warn, LogConfig};

fn main() {
    let config1 = LogConfig::new()
        .with_instance_name("app1")
        .with_path("C:/logs/")
        .with_file_name("app1.log")
        .with_max_files(5);

    let instance1 = rs_loglib::init_logger(config1).unwrap();

    let config2 = LogConfig::new()
        .with_instance_name("app2")
        .with_path("C:/logs/")
        .with_file_name("app2.log")
        .with_max_files(3);

    let instance2 = rs_loglib::init_logger(config2).unwrap();

    info!(instance1, "This is a message from app1");
    error!(instance1, "Error in app1");

    info!(instance2, "This is a message from app2");
    warn!(instance2, "Warning in app2");
}
