use std::{env, thread, ffi::OsString, sync::{Arc, atomic::{AtomicBool, Ordering}}, time::Duration};
use windows_service::{
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType, ServiceAccess, ServiceStartType, ServiceErrorControl, ServiceInfo,
        PowerEventParam,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_manager::{ServiceManager, ServiceManagerAccess},
};

use crate::config::AppConfig;
use crate::device::check_and_fix_network;

// Service Entry Point
pub fn my_service_main(_arguments: Vec<OsString>) {
    if let Err(e) = run_service() {
        log::error!("Service runtime error: {:?}", e);
    }
}

fn run_service() -> windows_service::Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let running_in_handler = running.clone();
    
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop | ServiceControl::Interrogate => {
                running_in_handler.store(false, Ordering::SeqCst);
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::PowerEvent(event_param) => {
                match event_param {
                    PowerEventParam::ResumeAutomatic | PowerEventParam::ResumeSuspend => {
                        log::info!("System wake detected (Automatic/Suspend).");
                        
                        thread::spawn(|| {
                             let wait_time = Duration::from_secs(AppConfig::global().wait_after_wake_secs);
                            log::info!("Waiting {:?} for network adapter initialization...", wait_time);
                            thread::sleep(wait_time);
                            // FORCE check because this is a wake event
                            check_and_fix_network(true);
                        });
                    }
                    _ => {}
                }
                ServiceControlHandlerResult::NoError
            }
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let config = AppConfig::global();
    let status_handle = service_control_handler::register(&config.service_name, event_handler)?;

    // Report Running state
    let next_status = ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::POWER_EVENT,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    };
    status_handle.set_service_status(next_status)?;
    
    log::info!("Service started successfully.");
    
    thread::spawn(|| {
        thread::sleep(Duration::from_secs(5));
        check_and_fix_network(false); 
    });

    // Main loop
    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_secs(60));
        check_and_fix_network(false);
    }
    
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;
    
    log::info!("Service stopped.");
    Ok(())
}

pub fn install_service() -> windows_service::Result<()> {
    let config = AppConfig::global();
    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_path = env::current_exe()
        .map_err(|e| windows_service::Error::Winapi(e))?;
    
    let service_info = ServiceInfo {
        name: OsString::from(&config.service_name),
        display_name: OsString::from(&config.service_display_name),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: service_path,
        launch_arguments: Vec::new(),
        dependencies: Vec::new(),
        account_name: None,
        account_password: None,
    };
    
    let service = service_manager.create_service(
        &service_info,
        ServiceAccess::START,
    )?;
    
    log::info!("Service created. Starting service...");
    let args: Vec<&str> = Vec::new();
    service.start(&args)?;
    log::info!("Service started.");

    Ok(())
}

pub fn uninstall_service() -> windows_service::Result<()> {
    let config = AppConfig::global();
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::DELETE | ServiceAccess::STOP;
    let service = service_manager.open_service(&config.service_name, service_access)?;

    // stop the service before deleting
    if let Err(e) = service.stop() {
        log::warn!("Failed to stop service (it might be already stopped): {}", e);
    } else {
        log::info!("Service stop signal sent.");
    }

    service.delete()?;
    Ok(())
}