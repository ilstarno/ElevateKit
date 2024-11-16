use std::ptr;
use winapi::um::winbase::CREATE_SUSPENDED;
use winapi::um::processthreadsapi::{
    PROCESS_INFORMATION,
    STARTUPINFOW,
    CreateProcessW,
    ResumeThread,
    GetExitCodeProcess,
};
use winapi::um::handleapi::CloseHandle;
use winapi::um::winnt::HANDLE;
use winapi::shared::minwindef::DWORD;
use winapi::um::minwinbase::STILL_ACTIVE;
use crate::utils::error::{ElevateResult, WindowsError, WindowsErrorKind};

/// Represents a Windows process with safe handle management
pub struct ProcessInfo {
    handle: HANDLE,
    thread_handle: HANDLE,
    process_id: u32,
    thread_id: u32,
}



impl ProcessInfo {
    pub fn process_id(&self) -> u32 {
        self.process_id
    }

    pub fn thread_id(&self) -> u32 {
        self.thread_id
    }
}

// Implement Drop for automatic handle cleanup
impl Drop for ProcessInfo {
    fn drop(&mut self) {
        unsafe {
            if !self.handle.is_null() {
                CloseHandle(self.handle);
            }
            if !self.thread_handle.is_null() {
                CloseHandle(self.thread_handle);
            }
        }
    }
}

impl From<PROCESS_INFORMATION> for ProcessInfo {
    fn from(info: PROCESS_INFORMATION) -> Self {
        Self {
            handle: info.hProcess,
            thread_handle: info.hThread,
            process_id: info.dwProcessId,
            thread_id: info.dwThreadId,
        }
    }
}

pub trait ProcessService {
    fn create_process(&self, command: &str) -> ElevateResult<ProcessInfo>;
    fn resume_process(&self, process: &ProcessInfo) -> ElevateResult<()>;
    fn wait_for_exit(&self, process: &ProcessInfo) -> ElevateResult<DWORD>;
}

pub struct WindowsProcessService;

impl WindowsProcessService {
    pub fn new() -> Self {
        Self
    }

    fn check_win32(result: i32, context: &str) -> ElevateResult<()> {
        if result == 0 {
            Err(WindowsError::last_error().to_elevate_error(
                WindowsErrorKind::ProcessOperation,
                context
            ))
        } else {
            Ok(())
        }
    }
}

impl ProcessService for WindowsProcessService {
    fn create_process(&self, command: &str) -> ElevateResult<ProcessInfo> {
        let mut startup_info: STARTUPINFOW = unsafe { std::mem::zeroed() };
        startup_info.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
        
        let mut process_info: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
        let wide_command: Vec<u16> = command.encode_utf16().chain(Some(0)).collect();
        
        let result = unsafe {
            CreateProcessW(
                ptr::null(),
                wide_command.as_ptr() as *mut u16,
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                CREATE_SUSPENDED,
                ptr::null_mut(),
                ptr::null(),
                &startup_info as *const STARTUPINFOW as *mut STARTUPINFOW,
                &mut process_info
            )
        };

        Self::check_win32(
            result,
            &format!("Failed to create process: {}", command)
        )?;

        Ok(process_info.into())
    }

    fn resume_process(&self, process: &ProcessInfo) -> ElevateResult<()> {
        let result = unsafe { ResumeThread(process.thread_handle) };
        if result == u32::MAX {
            return Err(WindowsError::last_error().to_elevate_error(
                WindowsErrorKind::ProcessOperation,
                "Failed to resume process thread"
            ));
        }
        Ok(())
    }

    fn wait_for_exit(&self, process: &ProcessInfo) -> ElevateResult<DWORD> {
        let mut exit_code = 0;
        let result = unsafe {
            GetExitCodeProcess(process.handle, &mut exit_code)
        };
        
        Self::check_win32(
            result,
            "Failed to get process exit code"
        )?;

        Ok(exit_code)
    }
}

// Enhanced process manager with better abstraction
pub struct ProcessManager {
    service: WindowsProcessService,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            service: WindowsProcessService::new()
        }
    }

    pub fn create_suspended_process(&self, command: &str) -> ElevateResult<ProcessInfo> {
        self.service.create_process(command)
    }

    pub fn execute_process(&self, command: &str) -> ElevateResult<DWORD> {
        let process = self.service.create_process(command)?;
        self.service.resume_process(&process)?;
        self.service.wait_for_exit(&process)
    }
}

// Process extension trait for additional functionality
pub trait ProcessExt {
    fn wait_for_exit(&self) -> ElevateResult<DWORD>;
    fn is_running(&self) -> ElevateResult<bool>;
}

impl ProcessExt for ProcessInfo {
    fn wait_for_exit(&self) -> ElevateResult<DWORD> {
        let mut exit_code = 0;
        let result = unsafe {
            GetExitCodeProcess(self.handle, &mut exit_code)
        };

        WindowsProcessService::check_win32(
            result,
            "Failed to get process exit code"
        )?;

        Ok(exit_code)
    }

    fn is_running(&self) -> ElevateResult<bool> {
        let mut exit_code = 0;
        let result = unsafe {
            GetExitCodeProcess(self.handle, &mut exit_code)
        };

        if result == 0 {
            return Err(WindowsError::last_error().to_elevate_error(
                WindowsErrorKind::ProcessOperation,
                "Failed to get process status"
            ));
        }

        Ok(exit_code == STILL_ACTIVE)
    }
}