use std::ffi;

use crate::errors::CGNSError;

pub const CGIO_MAX_NAME_LENGTH: usize = 32;
pub const CGIO_NAME_BUFFER_LENGTH: usize = CGIO_MAX_NAME_LENGTH + 1;

pub(crate) type Result<T = (), E = CGNSError> = ::core::result::Result<T, E>;

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
                let err = crate::errors::CGNSLibraryError(err_msg);
                Err(crate::errors::CGNSError::from(err))
            } else {
                Ok(())
            }
        }
    };
}

pub(crate) use ier_cg_fn;
