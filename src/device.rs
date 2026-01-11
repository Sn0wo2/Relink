use crate::config::AppConfig;
use std::thread;
use windows::core::HRESULT;
use windows::Win32::Devices::DeviceAndDriverInstallation::{
    SetupDiCallClassInstaller, SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo,
    SetupDiGetClassDevsW, SetupDiGetDeviceRegistryPropertyW, SetupDiSetClassInstallParamsW,
    DICS_DISABLE, DICS_ENABLE, DICS_FLAG_GLOBAL, DIF_PROPERTYCHANGE, DIGCF_ALLCLASSES,
    DIGCF_PRESENT, SP_CLASSINSTALL_HEADER, SP_DEVINFO_DATA, SP_PROPCHANGE_PARAMS,
    SPDRP_FRIENDLYNAME, SPDRP_DEVICEDESC, SETUP_DI_REGISTRY_PROPERTY, SETUP_DI_STATE_CHANGE,
};
use windows::Win32::Foundation::{GetLastError, ERROR_INVALID_DATA, NO_ERROR, ERROR_BUFFER_OVERFLOW};
use windows::Win32::NetworkManagement::IpHelper::{GetAdaptersAddresses, GAA_FLAG_INCLUDE_GATEWAYS, IP_ADAPTER_ADDRESSES_LH};
use std::time::Duration;

const ADAPTER_BUFFER_SIZE: u32 = 15000;
const MAX_ADAPTER_RETRIES: i32 = 3;
const BYTES_TO_MBPS_DIVISOR: u64 = 1_000_000;

pub fn get_link_speed(adapter_name: &str) -> Result<Option<u64>, windows::core::Error> {
    let mut out_buf_len: u32 = ADAPTER_BUFFER_SIZE;
    
    for _ in 0..MAX_ADAPTER_RETRIES {
        let mut p_addresses = vec![0u8; out_buf_len as usize];
        let p_adapter_addresses = p_addresses.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH;

        let dw_ret_val = unsafe {
            GetAdaptersAddresses(
                0, // AF_UNSPEC
                GAA_FLAG_INCLUDE_GATEWAYS,
                None,
                Some(p_adapter_addresses),
                &mut out_buf_len,
            )
        };

        if dw_ret_val == ERROR_BUFFER_OVERFLOW.0 {
            continue;
        }
        
        if dw_ret_val != NO_ERROR.0 {
            return Err(windows::core::Error::from_hresult(HRESULT::from_win32(dw_ret_val)));
        }

        let mut curr_ptr = p_adapter_addresses;
        while !curr_ptr.is_null() {
            let curr = unsafe { &*curr_ptr };
            let description = unsafe { curr.Description.to_string().unwrap_or_default() };
            let friendly_name = unsafe { curr.FriendlyName.to_string().unwrap_or_default() };

            if friendly_name.contains(adapter_name) || description.contains(adapter_name) {
                return Ok(Some(curr.ReceiveLinkSpeed));
            }
            curr_ptr = curr.Next;
        }
        break;
    }
    Ok(None)
}

pub unsafe fn restart_device_by_name(target_name: &str, restart_delay_secs: u64) -> windows::core::Result<bool> {
    // Safety check
    let dev_info = unsafe {
        SetupDiGetClassDevsW(
            None,
            None,
            None,
            DIGCF_ALLCLASSES | DIGCF_PRESENT,
        )?
    };

    if dev_info.is_invalid() {
        return Err(windows::core::Error::from_hresult(HRESULT::from_win32(unsafe { GetLastError().0 })));
    }

    let mut dev_info_data = SP_DEVINFO_DATA {
        cbSize: size_of::<SP_DEVINFO_DATA>() as u32,
        ..Default::default()
    };

    let mut i = 0;
    let mut found = false;

    // Safety check
    while unsafe { SetupDiEnumDeviceInfo(dev_info, i, &mut dev_info_data).is_ok() } {
        i += 1;

        let name_res = get_device_property(dev_info, &mut dev_info_data, SPDRP_FRIENDLYNAME)
            .or_else(|_| get_device_property(dev_info, &mut dev_info_data, SPDRP_DEVICEDESC));

        if let Ok(name) = name_res {
            if name == target_name {
                log::info!("Device found: {}", name);
                found = true;

                log::info!("Disabling device...");
                set_device_state(dev_info, &mut dev_info_data, DICS_DISABLE)?;
                
                thread::sleep(Duration::from_secs(restart_delay_secs));

                log::info!("Enabling device...");
                set_device_state(dev_info, &mut dev_info_data, DICS_ENABLE)?;
                
                break;
            }
        }
    }

    // Safety check
    unsafe { SetupDiDestroyDeviceInfoList(dev_info)? };
    Ok(found)
}

unsafe fn get_device_property(
    dev_info: windows::Win32::Devices::DeviceAndDriverInstallation::HDEVINFO,
    dev_info_data: &mut SP_DEVINFO_DATA,
    property: SETUP_DI_REGISTRY_PROPERTY,
) -> Result<String, windows::core::Error> {
    let mut required_size = 0;
    // Safety
    let _ = unsafe {
        SetupDiGetDeviceRegistryPropertyW(
            dev_info,
            dev_info_data,
            property,
            None,
            None,
            Some(&mut required_size),
        )
    };

    if required_size == 0 {
        return Err(windows::core::Error::from_hresult(HRESULT::from_win32(ERROR_INVALID_DATA.0)));
    }

    let mut buffer = vec![0u8; required_size as usize];
    // Safety
    unsafe {
        SetupDiGetDeviceRegistryPropertyW(
            dev_info,
            dev_info_data,
            property,
            None,
            Some(&mut buffer),
            Some(&mut required_size),
        )?
    };

    let wide_buffer: Vec<u16> = buffer
        .chunks_exact(2)
        .map(|chunk| u16::from_ne_bytes([chunk[0], chunk[1]]))
        .collect();

    let len = wide_buffer.iter().position(|&x| x == 0).unwrap_or(wide_buffer.len());
    Ok(String::from_utf16_lossy(&wide_buffer[..len]))
}

unsafe fn set_device_state(
    dev_info: windows::Win32::Devices::DeviceAndDriverInstallation::HDEVINFO,
    dev_info_data: &mut SP_DEVINFO_DATA,
    state: SETUP_DI_STATE_CHANGE,
) -> windows::core::Result<()> {
    let class_install_header = SP_CLASSINSTALL_HEADER {
        cbSize: std::mem::size_of::<SP_CLASSINSTALL_HEADER>() as u32,
        InstallFunction: DIF_PROPERTYCHANGE,
    };

    let mut prop_change_params = SP_PROPCHANGE_PARAMS {
        ClassInstallHeader: class_install_header,
        StateChange: state,
        Scope: DICS_FLAG_GLOBAL,
        HwProfile: 0,
    };

    // Safety
    unsafe {
        SetupDiSetClassInstallParamsW(
            dev_info,
            Some(dev_info_data),
            Some(&mut prop_change_params as *mut _ as *mut _),
            size_of::<SP_PROPCHANGE_PARAMS>() as u32,
        )?
    };

    // Safety
    unsafe {
        SetupDiCallClassInstaller(
            DIF_PROPERTYCHANGE,
            dev_info,
            Some(dev_info_data),
        )
    }
}

pub fn check_and_fix_network(force_check: bool) {
    let config = AppConfig::global();
    let target_adapter = &config.target_adapter_name;
    let threshold = config.link_speed_threshold_bps;
    let restart_delay = config.restart_delay_secs;

    if force_check {
        log::info!("Performing forced network check (e.g., after wake)...");
    } else {
        log::info!("Performing routine network check...");
    }

    match get_link_speed(target_adapter) {
        Ok(Some(speed)) => {
            let speed_mbps = speed / BYTES_TO_MBPS_DIVISOR;
            log::info!("Current Link Speed: {} Mbps", speed_mbps);
            
            if speed <= threshold {
                 if force_check {
                     log::warn!("Speed detected as <= {} Mbps AFTER WAKE. Initiating restart sequence.", threshold / BYTES_TO_MBPS_DIVISOR);
                     match unsafe { restart_device_by_name(target_adapter, restart_delay) } {
                        Ok(true) => log::info!("Device restart sequence completed successfully."),
                        Ok(false) => log::error!("Device '{}' not found.", target_adapter),
                        Err(e) => log::error!("Failed to restart device: {:?}", e),
                    }
                 } else {
                     log::warn!("Speed detected as <= {} Mbps, but not a wake event. Ignoring to prevent random restarts during normal use.", threshold / BYTES_TO_MBPS_DIVISOR);
                 }
            } else {
                log::info!("Speed is normal (>{} Mbps). No action required.", threshold / BYTES_TO_MBPS_DIVISOR);
            }
        }
        Ok(None) => log::error!("Adapter '{}' not found in network interfaces.", target_adapter),
        Err(e) => log::error!("Failed to retrieve adapter info: {:?}", e),
    }
}