use std::env;
use std::fs::File;
use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, WriteLogger, TerminalMode, ColorChoice, SharedLogger};

pub fn init_logger() {
    let mut path = env::current_exe().unwrap_or_default();
    path.set_file_name("relink_service.log");

    let file = File::options().create(true).append(true).open(path).ok();

    let mut loggers: Vec<Box<dyn SharedLogger>> = Vec::new();

    loggers.push(TermLogger::new(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    ));

    if let Some(f) = file {
        loggers.push(WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            f,
        ));
    } else {
        eprintln!("Failed to open log file for writing.");
    }
    
    let _ = CombinedLogger::init(loggers);
}