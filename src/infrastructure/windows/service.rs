use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::ptr;
use std::mem;
use std::thread;
use std::time::Duration;
use winapi::um::winsvc::*;
use winapi::shared::winerror::*;
use log::{debug, info, error};

use crate::infrastructure::error::{TiError, TiResult};
use crate::domain::constants::SERVICE_NAME;

pub struct WindowsService;

impl WindowsService {
    pub fn get_trusted_installer_pid() -> TiResult<u32> {
        let manager = unsafe { OpenSCManagerW(ptr::null(), ptr::null(), SC_MANAGER_CONNECT) };
        if manager.is_null() {
            return Err(TiError::ServiceError("Failed to open service control manager".into()));
        }

        let service_name: Vec<u16> = OsString::from(SERVICE_NAME)
            .encode_wide()
            .chain(Some(0))
            .collect();

        let service = unsafe { 
            OpenServiceW(
                manager,
                service_name.as_ptr(),
                SERVICE_START | SERVICE_QUERY_STATUS
            )
        };

        if service.is_null() {
            unsafe { CloseServiceHandle(manager) };
            return Err(TiError::ServiceError(
                "Failed to open TrustedInstaller service (are you running as Administrator?)".into()
            ));
        }

        // Start service if not running
        let start_result = unsafe { StartServiceW(service, 0, ptr::null_mut()) };
        if start_result == 0 {
            let error = unsafe { winapi::um::errhandlingapi::GetLastError() };
            if error != ERROR_SERVICE_ALREADY_RUNNING {
                unsafe {
                    CloseServiceHandle(service);
                    CloseServiceHandle(manager);
                }
                return Err(TiError::ServiceError("Failed to start TrustedInstaller service".into()));
            }
        }

        // Query service status
        let mut status: SERVICE_STATUS_PROCESS = unsafe { mem::zeroed() };
        let mut bytes_needed: u32 = 0;

        loop {
            let query_result = unsafe {
                QueryServiceStatusEx(
                    service,
                    SC_STATUS_PROCESS_INFO,
                    &mut status as *mut _ as *mut u8,
                    mem::size_of::<SERVICE_STATUS_PROCESS>() as u32,
                    &mut bytes_needed
                )
            };

            if query_result == 0 {
                unsafe {
                    CloseServiceHandle(service);
                    CloseServiceHandle(manager);
                }
                return Err(TiError::ServiceError("Failed to query service status".into()));
            }

            if status.dwCurrentState == SERVICE_RUNNING {
                break;
            }

            if status.dwCurrentState != SERVICE_START_PENDING {
                unsafe {
                    CloseServiceHandle(service);
                    CloseServiceHandle(manager);
                }
                return Err(TiError::ServiceError("Service is not running or starting".into()));
            }

            thread::sleep(Duration::from_millis(100));
        }

        let pid = status.dwProcessId;

        unsafe {
            CloseServiceHandle(service);
            CloseServiceHandle(manager);
        }

        Ok(pid)
    }
} 