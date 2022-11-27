use std::ffi;

use anyhow::Result;

pub const CGIO_MAX_NAME_LENGTH: usize = 32;
pub const CGIO_NAME_BUFFER_LENGTH: usize = CGIO_MAX_NAME_LENGTH + 1;

#[derive(Debug, Clone, thiserror::Error)]
pub struct CGNSError {
    msg: String,
}

impl std::fmt::Display for CGNSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CGNS: {}", self.msg)
    }
}

impl CGNSError {
    pub fn new<T>(msg: T) -> Self
    where
        T: Into<String> + Sized,
    {
        Self { msg: msg.into() }
    }
}

pub(crate) fn bytes2string(bytes: &[u8]) -> Result<String> {
    // TODO: use ffi::CStr::from_bytes_until_nul once it's stabilized
    let null_byte = bytes
        .iter()
        .position(|&e| e == 0)
        .unwrap_or(bytes.len() - 1);
    let bytes = &bytes[0..null_byte + 1];
    Ok(ffi::CStr::from_bytes_with_nul(bytes)?.to_str()?.to_owned())
}

/// EZ wrapper for CGNS functions that return `ier`
macro_rules! ier_cg_fn {
    ($func_call:expr) => {
        unsafe {
            let err_code = $func_call;
            if err_code != i32::try_from(CG_OK).unwrap() {
                let err_msg = cg_get_error();
                if err_msg.is_null() {
                    Err(anyhow!(CGNSError::new("Unknown CGNS ERROR")))
                } else {
                    let err_msg = ffi::CStr::from_ptr(err_msg);
                    let err_msg = err_msg.to_str().unwrap_or("(invalid error string)");
                    Err(anyhow!(CGNSError::new(err_msg)))
                }
            } else {
                Ok(())
            }
        }
    };
}

pub(crate) use ier_cg_fn;
