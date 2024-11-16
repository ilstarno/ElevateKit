use std::ptr;
use winapi::um::winnt::{TOKEN_PRIVILEGES, SE_PRIVILEGE_ENABLED, LUID};
use winapi::um::winbase::LookupPrivilegeValueW;
use winapi::um::securitybaseapi::AdjustTokenPrivileges;
use crate::utils::error::{ElevateError, ElevateResult, PrivilegeErrorKind, TokenErrorKind};
use log::info;

pub struct PrivilegeManager {
    token_handle: winapi::um::winnt::HANDLE,
}

impl PrivilegeManager {
    pub fn new(token_handle: winapi::um::winnt::HANDLE) -> Self {
        Self { token_handle }
    }

    pub fn enable(&self, privilege: &str) -> ElevateResult<()> {
        info!("Enabling privilege: {}", privilege);
        self.adjust_privilege(privilege, true)
    }

    pub fn disable(&self, privilege: &str) -> ElevateResult<()> {
        info!("Disabling privilege: {}", privilege);
        self.adjust_privilege(privilege, false)
    }

    fn adjust_privilege(&self, privilege: &str, enable: bool) -> ElevateResult<()> {
        let mut luid = LUID { LowPart: 0, HighPart: 0 };
        let name_wide: Vec<u16> = privilege.encode_utf16().chain(Some(0)).collect();

        let success = unsafe {
            LookupPrivilegeValueW(
                ptr::null(),
                name_wide.as_ptr(),
                &mut luid
            )
        };

        if success == 0 {
            return Err(ElevateError::TokenError {
                kind: TokenErrorKind::LookupPrivilege,
                source: None,
            });
        }

        let mut tp = TOKEN_PRIVILEGES {
            PrivilegeCount: 1,
            Privileges: [unsafe { std::mem::zeroed() }],
        };
        tp.Privileges[0].Luid = luid;
        tp.Privileges[0].Attributes = if enable { SE_PRIVILEGE_ENABLED } else { 0 };

        let success = unsafe {
            AdjustTokenPrivileges(
                self.token_handle,
                0,
                &tp as *const TOKEN_PRIVILEGES as *mut TOKEN_PRIVILEGES,
                0,
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };

        if success == 0 {
            return Err(ElevateError::PrivilegeError {
                kind: if enable { PrivilegeErrorKind::Enable } else { PrivilegeErrorKind::Disable },
                privilege: Some(privilege.to_string()),
                source: None,
            });
        }

        Ok(())
    }
}
