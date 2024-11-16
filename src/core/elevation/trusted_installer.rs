use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use tracing::info;
use winapi::{
    shared::{minwindef::DWORD, winerror::ERROR_SERVICE_ALREADY_RUNNING},
    um::{
        errhandlingapi::GetLastError, heapapi::{GetProcessHeap, HeapAlloc, HeapFree}, processthreadsapi::*, winbase::*, winnt::PROCESS_CREATE_PROCESS, winsvc::*
    },
};

use crate::utils::{error::{ElevateError, ElevateResult}, security::SecurityContext};

const PROC_THREAD_ATTRIBUTE_PARENT_PROCESS: DWORD = 0x00020000;

// handle wrapper for windows handles
struct HandleGuard<T>(pub *mut T);

impl<T> Drop for HandleGuard<T> {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { winapi::um::handleapi::CloseHandle(self.0 as _) };
        }
    }
}

pub struct TrustedInstallerElevation<'a> {
    context: &'a SecurityContext,
}

impl<'a> TrustedInstallerElevation<'a> {
    pub fn new(context: &'a SecurityContext) -> Self {
        Self { context }
    }

    pub fn execute(&self, command: &str, args: &[String]) -> ElevateResult<()> {
        info!("Elevating to TrustedInstaller");
        
        // get trustedinstaller pid
        let ti_pid = Self::get_trusted_installer_pid()?;
        info!("TrustedInstaller PID: {}", ti_pid);

        // create process with ti parent
        let proc_info = Self::create_process_with_ti_parent(ti_pid, command, args)?;
        
        // set privileges and resume
        unsafe {
            self.context.set_all_privileges(proc_info.hProcess)
                .map_err(|e| ElevateError::ProcessError(e.to_string()))?;
            ResumeThread(proc_info.hThread);
            
            // process handles cleaned up by handleguard
            let _proc_guard = HandleGuard(proc_info.hProcess);
            let _thread_guard = HandleGuard(proc_info.hThread);
        }

        Ok(())
    }

    fn get_trusted_installer_pid() -> ElevateResult<u32> {
        unsafe {
            let scm = HandleGuard(OpenSCManagerW(null_mut(), null_mut(), SC_MANAGER_CONNECT));
            if scm.0.is_null() {
                return Err(ElevateError::ProcessError("Failed to open service control manager".into()));
            }
    
            let service = HandleGuard(OpenServiceW(
                scm.0,
                to_wide_str("TrustedInstaller").as_ptr(),
                SERVICE_START | SERVICE_QUERY_STATUS
            ));
    
            if service.0.is_null() {
                return Err(ElevateError::ProcessError(
                    "Failed to open TrustedInstaller service (are you running as Administrator?)".into()
                ));
            }
    
            // try starting service
            if StartServiceW(service.0, 0, null_mut()) == 0 {
                let err = GetLastError();
                if err != ERROR_SERVICE_ALREADY_RUNNING {
                    return Err(ElevateError::ProcessError("Failed to start TrustedInstaller service".into()));
                }
            }
    
            // wait up to 10 seconds for service to run
            for _ in 0..20 {
                let mut status: SERVICE_STATUS_PROCESS = std::mem::zeroed();
                let mut bytes_needed = 0;
    
                if QueryServiceStatusEx(
                    service.0,
                    SC_STATUS_PROCESS_INFO,
                    &mut status as *mut _ as *mut u8,
                    std::mem::size_of::<SERVICE_STATUS_PROCESS>() as u32,
                    &mut bytes_needed
                ) == 0 {
                    return Err(ElevateError::ProcessError("Failed to query service status".into()));
                }
    
                match status.dwCurrentState {
                    SERVICE_RUNNING => return Ok(status.dwProcessId),
                    SERVICE_START_PENDING => {
                        // wait 500ms before checking again
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        continue;
                    }
                    _ => return Err(ElevateError::ProcessError(
                        format!("Service in unexpected state: {}", status.dwCurrentState).into()
                    ))
                }
            }
    
            Err(ElevateError::ProcessError("Timeout waiting for service to start".into()))
        }
    }

    fn create_process_with_ti_parent(
        ti_pid: u32,
        command: &str,
        args: &[String]
    ) -> ElevateResult<PROCESS_INFORMATION> {
        unsafe {
            // open ti process
            let ti_handle = HandleGuard(OpenProcess(PROCESS_CREATE_PROCESS, 0, ti_pid));
            if ti_handle.0.is_null() {
                return Err(ElevateError::ProcessError("Failed to open TI process".into()));
            }

            // setup process attribute list
            let mut size = 0;
            InitializeProcThreadAttributeList(null_mut(), 1, 0, &mut size);
            let attr_list = HeapAlloc(GetProcessHeap(), 0, size) as *mut PROC_THREAD_ATTRIBUTE_LIST;
            if attr_list.is_null() {
                return Err(ElevateError::ProcessError("Failed to allocate attribute list".into()));
            }

            InitializeProcThreadAttributeList(attr_list, 1, 0, &mut size);
            UpdateProcThreadAttribute(
                attr_list,
                0,
                PROC_THREAD_ATTRIBUTE_PARENT_PROCESS.try_into().unwrap(),
                &ti_handle.0 as *const _ as *mut _,
                std::mem::size_of::<winapi::shared::ntdef::HANDLE>(),
                null_mut(),
                null_mut()
            );

            // setup startup info
            let mut startup_info = STARTUPINFOEXW {
                StartupInfo: STARTUPINFOW {
                    cb: std::mem::size_of::<STARTUPINFOEXW>() as u32,
                    ..std::mem::zeroed()
                },
                lpAttributeList: attr_list,
            };

            // make command line
            let cmd = format!("{} {}", command, args.join(" ")).trim().to_string();
            let cmd_wide = to_wide_str(&cmd);

            // create process
            let mut process_info: PROCESS_INFORMATION = std::mem::zeroed();
            let result = CreateProcessW(
                null_mut(),
                cmd_wide.as_ptr() as *mut _,
                null_mut(),
                null_mut(),
                0,
                CREATE_SUSPENDED | EXTENDED_STARTUPINFO_PRESENT | CREATE_NEW_CONSOLE,
                null_mut(),
                null_mut(),
                &mut startup_info.StartupInfo,
                &mut process_info
            );

            // cleanup
            DeleteProcThreadAttributeList(attr_list);
            HeapFree(GetProcessHeap(), 0, attr_list as *mut _);

            if result == 0 {
                return Err(ElevateError::ProcessError("Failed to create process".into()));
            }

            Ok(process_info)
        }
    }
}

fn to_wide_str(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}