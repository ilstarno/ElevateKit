use winapi::um::winnt::HANDLE;
use winapi::um::processthreadsapi::PROCESS_INFORMATION;

#[derive(Debug)]
pub struct ProcessInfo {
    pub handle: HANDLE,
    pub thread_handle: HANDLE,
    pub process_id: u32,
    pub thread_id: u32,
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

#[derive(Debug)]
pub struct TokenHandle(pub HANDLE);

impl Drop for TokenHandle {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { winapi::um::handleapi::CloseHandle(self.0) };
        }
    }
}

impl From<PROCESS_INFORMATION> for TokenHandle {
    fn from(info: PROCESS_INFORMATION) -> Self {
        Self(info.hProcess)
    }
}

#[derive(Debug)]
pub struct ServiceInfo {
    pub handle: HANDLE,
    pub process_id: u32,
    pub status: u32,
}

#[derive(Debug, Clone)]
pub struct PrivilegeInfo {
    pub name: String,
    pub enabled: bool,
    pub description: Option<String>,
}
 