use std::ptr;
use winapi::um::winnt::{
    TOKEN_ADJUST_PRIVILEGES, TOKEN_QUERY, HANDLE
};
use winapi::um::processthreadsapi::{OpenProcessToken, GetCurrentProcess};
use crate::utils::error::{ElevateError, ElevateResult, TokenErrorKind};

pub struct TokenManager {
    handle: HANDLE,
}

impl TokenManager {
    pub fn new() -> ElevateResult<Self> {
        let mut token = ptr::null_mut();
        
        let success = unsafe {
            OpenProcessToken(
                GetCurrentProcess(),
                TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
                &mut token,
            )
        };

        if success == 0 {
            return Err(ElevateError::TokenError {
                kind: TokenErrorKind::OpenProcessToken,
                source: None,
            });
        }

        Ok(Self { handle: token })
    }
}

impl Drop for TokenManager {
    fn drop(&mut self) {
        unsafe {
            winapi::um::handleapi::CloseHandle(self.handle);
        }
    }
}
