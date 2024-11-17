use anyhow::Result;
use libc;
use std::{error::Error, ffi::CString, fmt::Display};

#[derive(Debug)]
pub struct SyscallError {
    code: i64,
}
impl Display for SyscallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Syscall error: code {}", self.code)
    }
}
impl Error for SyscallError {
    fn description(&self) -> &str {
        return "Syscall error!";
    }
}

pub unsafe fn pivot_root(target: &str, put_old: &str) -> Result<i64> {
    let code = libc::syscall(
        libc::SYS_pivot_root,
        CString::new(target)?.as_ptr(),
        CString::new(put_old)?.as_ptr(),
    );
    if code != 0 {
        Err(SyscallError { code }.into())
    } else {
        Ok(0)
    }
}
