use std::ffi::OsString;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;
use std::ptr;
use std::mem;
use winapi::um::winnt::{TOKEN_ADJUST_PRIVILEGES, TOKEN_QUERY, SE_PRIVILEGE_ENABLED, LUID, TOKEN_PRIVILEGES, HANDLE};
use winapi::um::securitybaseapi::{AdjustTokenPrivileges, ImpersonateSelf};
use winapi::um::winbase::LookupPrivilegeValueW;
use winapi::um::processthreadsapi::{OpenThreadToken, GetCurrentThread};
use winapi::um::winnt::SecurityImpersonation;
use log::{debug, error, info};

use crate::domain::models::{PrivilegeInfo, TokenHandle};
use crate::domain::constants::ALL_TOKEN_PRIVILEGES;
use crate::infrastructure::error::{TiError, TiResult};

// defines the interface for managing Windows privileges
pub trait PrivilegeService {
    fn set_privilege(&self, name: &str) -> TiResult<()>;
    fn set_all_privileges(&self, token: TokenHandle) -> TiResult<()>;
    fn get_privileges(&self) -> TiResult<Vec<PrivilegeInfo>>;
    fn enable_debug_privilege(&self) -> TiResult<()>;
}

pub struct WindowsPrivilegeService;

impl WindowsPrivilegeService {
    pub fn new() -> Self {
        Self {}
    }

    // gets a token for the current thread that can be used to adjust privileges
    fn get_thread_token(&self) -> TiResult<TokenHandle> {
        unsafe { 
            // impersonate the current user to get a token
            ImpersonateSelf(SecurityImpersonation);
            let mut token = ptr::null_mut();
            if OpenThreadToken(GetCurrentThread(), TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY, 0, &mut token) == 0 {
                error!("Failed to open thread token");
                return Err(TiError::PrivilegeError("Failed to open thread token".into()));
            }
            Ok(TokenHandle(token))
        }
    }

    // enables or disables a specific privilege on a token
    fn adjust_token_privilege(&self, token: HANDLE, name: &str, enable: bool) -> TiResult<()> {
        // convert privilege name to wide string format for windows api
        let name_w: Vec<u16> = OsString::from(name).encode_wide().chain(Some(0)).collect();
        
        // get the LUID (locally unique identifier) for the privilege
        let mut luid: LUID = unsafe { mem::zeroed() };
        let lookup_result = unsafe { 
            LookupPrivilegeValueW(ptr::null(), name_w.as_ptr(), &mut luid)
        };

        // handle errors when looking up the privilege
        if lookup_result == 0 {
            let error = unsafe { winapi::um::errhandlingapi::GetLastError() };
            if error == 1313 { // ERROR_NO_SUCH_PRIVILEGE
                debug!("Privilege {} not found on this system", name);
                return Ok(());
            }
            error!("Failed to lookup privilege value for {}", name);
            return Err(TiError::PrivilegeError(format!("Failed to lookup privilege: {}", name)));
        }

        // prepare the privilege structure for adjustment
        let mut privs: TOKEN_PRIVILEGES = unsafe { mem::zeroed() };
        privs.PrivilegeCount = 1;
        privs.Privileges[0].Luid = luid;
        privs.Privileges[0].Attributes = if enable { SE_PRIVILEGE_ENABLED } else { 0 };
        
        // attempt to adjust the privilege
        if unsafe { AdjustTokenPrivileges(token, 0, &mut privs, 0, ptr::null_mut(), ptr::null_mut()) } == 0 {
            error!("Failed to adjust token privileges for {}", name);
            return Err(TiError::PrivilegeError(format!("Failed to adjust privilege: {}", name)));
        }

        debug!("Successfully {} privilege {}", if enable { "enabled" } else { "disabled" }, name);
        Ok(())
    }
}

impl PrivilegeService for WindowsPrivilegeService {
    // enables a single privilege by name
    fn set_privilege(&self, name: &str) -> TiResult<()> {
        let token = self.get_thread_token()?;
        self.adjust_token_privilege(token.0, name, true)
    }

    // attempts to enable all available privileges on a token
    fn set_all_privileges(&self, token: TokenHandle) -> TiResult<()> {
        info!("Setting all available privileges");
        for privilege in ALL_TOKEN_PRIVILEGES.iter() {
            if let Err(e) = self.adjust_token_privilege(token.0, privilege, true) {
                debug!("Failed to set privilege {}: {}", privilege, e);
                // continue with other privileges even if one fails
                continue;
            }
        }
        Ok(())
    }

    // gets a list of all privileges and their status
    fn get_privileges(&self) -> TiResult<Vec<PrivilegeInfo>> {
        let token = self.get_thread_token()?;
        let mut privileges = Vec::new();
        
        // check each privilege in the predefined list
        for name in ALL_TOKEN_PRIVILEGES.iter() {
            let name_w: Vec<u16> = OsString::from(*name).encode_wide().chain(Some(0)).collect();
            let mut luid: LUID = unsafe { mem::zeroed() };
            
            let exists = unsafe { 
                LookupPrivilegeValueW(ptr::null(), name_w.as_ptr(), &mut luid) != 0 
            };

            privileges.push(PrivilegeInfo {
                name: name.to_string(),
                enabled: exists,
                description: None,
            });
        }

        Ok(privileges)
    }

    // convenience method to enable the debug privilege specifically
    fn enable_debug_privilege(&self) -> TiResult<()> {
        info!("Enabling debug privilege");
        self.set_privilege("SeDebugPrivilege")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privilege_service() {
        let service = WindowsPrivilegeService::new();
        
        // test getting privileges
        let privileges = service.get_privileges().unwrap();
        assert!(!privileges.is_empty());
        
        // test enabling debug privilege
        assert!(service.enable_debug_privilege().is_ok());
    }
} 