use std::ptr;
use winapi::um::winnt::{HANDLE, TOKEN_ADJUST_PRIVILEGES, TOKEN_QUERY, SE_PRIVILEGE_ENABLED, TOKEN_ALL_ACCESS};
use winapi::um::processthreadsapi::{CreateProcessW, OpenProcessToken, GetCurrentProcess};
use winapi::um::securitybaseapi::ImpersonateSelf;
use winapi::um::winnt::SecurityImpersonation;
use winapi::um::processthreadsapi::{OpenThreadToken, GetCurrentThread};
use winapi::um::winbase::{CREATE_SUSPENDED, CREATE_NEW_CONSOLE};
use winapi::shared::minwindef::DWORD;

pub struct SecurityContext {
    handle: HANDLE,
}

impl SecurityContext {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Enable debug privilege first
        Self::set_debug_privilege()?;
        
        // Get current process token
        let mut token = ptr::null_mut();
        let current_process = unsafe { GetCurrentProcess() };
        
        if unsafe { OpenProcessToken(current_process, TOKEN_ALL_ACCESS, &mut token) } == 0 {
            return Err("Failed to open process token".into());
        }

        Ok(SecurityContext { handle: token })
    }

    fn set_debug_privilege() -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            // Impersonate self
            if ImpersonateSelf(SecurityImpersonation) == 0 {
                return Err("Failed to impersonate self".into());
            }

            // Get thread token
            let mut token = ptr::null_mut();
            if OpenThreadToken(
                GetCurrentThread(),
                TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
                0,
                &mut token
            ) == 0 {
                return Err("Failed to open thread token".into());
            }

            // Set debug privilege
            Self::set_token_privilege(token, "SeDebugPrivilege")?;
        }
        Ok(())
    }

    pub fn create_process(&self, command: &str, suspended: bool) -> Result<Process, Box<dyn std::error::Error>> {
        let mut wide_command: Vec<u16> = command.encode_utf16().chain(std::iter::once(0)).collect();
        let mut startup_info = unsafe { std::mem::zeroed::<winapi::um::processthreadsapi::STARTUPINFOW>() };
        startup_info.cb = std::mem::size_of::<winapi::um::processthreadsapi::STARTUPINFOW>() as u32;
        
        let mut process_info = unsafe { std::mem::zeroed::<winapi::um::processthreadsapi::PROCESS_INFORMATION>() };
        
        let creation_flags = if suspended { 
            CREATE_SUSPENDED | CREATE_NEW_CONSOLE 
        } else { 
            CREATE_NEW_CONSOLE 
        };

        let success = unsafe {
            CreateProcessW(
                ptr::null(),
                wide_command.as_mut_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                creation_flags,
                ptr::null_mut(),
                ptr::null(),
                &mut startup_info,
                &mut process_info
            )
        };

        if success == 0 {
            return Err("Failed to create process".into());
        }

        Ok(Process::new(process_info))
    }

    fn set_token_privilege(token: HANDLE, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        use winapi::um::winbase::LookupPrivilegeValueW;
        use winapi::um::securitybaseapi::AdjustTokenPrivileges;
        
        let mut luid = unsafe { std::mem::zeroed::<winapi::um::winnt::LUID>() };
        let name_wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();

        unsafe {
            if LookupPrivilegeValueW(
                ptr::null(),
                name_wide.as_ptr(),
                &mut luid
            ) == 0 {
                return Err("Failed to lookup privilege value".into());
            }

            let mut tp = winapi::um::winnt::TOKEN_PRIVILEGES {
                PrivilegeCount: 1,
                Privileges: [winapi::um::winnt::LUID_AND_ATTRIBUTES {
                    Luid: luid,
                    Attributes: SE_PRIVILEGE_ENABLED,
                }],
            };

            if AdjustTokenPrivileges(
                token,
                0,
                &mut tp,
                0,
                ptr::null_mut(),
                ptr::null_mut()
            ) == 0 {
                return Err("Failed to adjust token privileges".into());
            }
        }

        Ok(())
    }

    pub fn set_all_privileges(&self, process_handle: HANDLE) -> Result<(), Box<dyn std::error::Error>> {
        // Enable all privileges for the process
        unsafe {
            let token = self.get_process_token(process_handle, TOKEN_ADJUST_PRIVILEGES)?;
            self.enable_all_privileges(token.0)
        }
    }

    fn get_process_token(&self, process_handle: HANDLE, access: DWORD) -> Result<HandleWrapper, Box<dyn std::error::Error>> {
        let mut token = ptr::null_mut();
        if unsafe { OpenProcessToken(process_handle, access, &mut token) } == 0 {
            return Err("Failed to open process token".into());
        }
        Ok(HandleWrapper(token))
    }

    fn enable_all_privileges(&self, token: HANDLE) -> Result<(), Box<dyn std::error::Error>> {
        // Common privilege names that might be useful
        let privileges = [
            "SeDebugPrivilege",
            "SeBackupPrivilege",
            "SeRestorePrivilege",
            "SeShutdownPrivilege",
            "SeSystemtimePrivilege",
        ];

        for privilege in privileges.iter() {
            if let Err(_) = Self::set_token_privilege(token, privilege) {
                // Ignore individual privilege errors
                continue;
            }
        }
        Ok(())
    }
}

pub struct Process {
    handle: HANDLE,
    thread_handle: HANDLE,
    process_id: DWORD,
    thread_id: DWORD,
}

impl Process {
    pub fn new(info: winapi::um::processthreadsapi::PROCESS_INFORMATION) -> Self {
        Self {
            handle: info.hProcess,
            thread_handle: info.hThread,
            process_id: info.dwProcessId,
            thread_id: info.dwThreadId,
        }
    }

    pub fn resume(&self) -> Result<(), Box<dyn std::error::Error>> {
        if unsafe { winapi::um::processthreadsapi::ResumeThread(self.thread_handle) } == u32::MAX {
            return Err("Failed to resume thread".into());
        }
        Ok(())
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        unsafe {
            if !self.handle.is_null() {
                winapi::um::handleapi::CloseHandle(self.handle);
            }
            if !self.thread_handle.is_null() {
                winapi::um::handleapi::CloseHandle(self.thread_handle);
            }
        }
    }
}

// Add this struct to handle automatic cleanup of Windows handles
struct HandleWrapper(HANDLE);

impl Drop for HandleWrapper {
    fn drop(&mut self) {
        unsafe {
            if !self.0.is_null() {
                winapi::um::handleapi::CloseHandle(self.0);
            }
        }
    }
}