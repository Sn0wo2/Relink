mod config;
mod device;
mod logger;
mod service;

use std::env;
use std::io::stdin;
use windows_service::{
    define_windows_service,
    service_dispatcher,
};

use crate::config::AppConfig;
use crate::service::{my_service_main, install_service, uninstall_service};
use crate::logger::init_logger;

define_windows_service!(ffi_service_main, my_service_main);


fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize config and logger early
    AppConfig::init();
    init_logger();

    let args: Vec<String> = env::args().collect();
    let config = AppConfig::global();
    
    // Command Line Interface
    if args.len() > 1 {
        match args[1].as_str() {
            "install" => {
                log::info!("Installing service '{}'...", config.service_display_name);
                if let Err(e) = install_service() {
                    log::error!("Failed to install service: {}", e);
                } else {
                    log::info!("Success! Service installed.");
                }
                // Config file creation if not exists
                if !AppConfig::get_path().exists() {
                    log::info!("Creating default config file...");
                    if let Err(e) = config.save() {
                        log::error!("Failed to save default config: {}", e);
                    } else {
                        log::info!("Default config created at {:?}", AppConfig::get_path());
                    }
                }
                
                log::info!("Press Enter to exit...");
                let mut s = String::new();
                stdin().read_line(&mut s)?;
            }
            "uninstall" => {
                log::info!("Uninstalling service '{}'...", config.service_display_name);
                uninstall_service()?;
                log::info!("Success! Service uninstalled.");
                
                log::info!("Press Enter to exit...");
                let mut s = String::new();
                stdin().read_line(&mut s)?;
            }
            _ => {
                print_usage();
            }
        }
        return Ok(());
    }

    // Service Mode
    // Attempt to start the service dispatcher
    // Since init_logger is already called, logs will go to file (and std which service ignores/redirects)
    match service_dispatcher::start(&config.service_name, ffi_service_main) {
        Ok(_) => {},
        Err(e) => {
            log::error!("Failed to start service dispatcher: {}", e);
            // If we are running in console but not as service, this error will show up.
            // But since we are using the TeeLogger, it will also show in stdout.
            println!("Hint: This program is a Windows Service. Run with 'install' to register it.");
        }
    }
    Ok(())
}

fn print_usage() {
    println!("Relink Network Monitor Service");
    println!("Usage:");
    println!("  relink install   - Install the service (Requires Admin)");
    println!("  relink uninstall - Uninstall the service (Requires Admin)");
    println!("  [No Arguments]   - Run as service (Called by SCM)");
}
