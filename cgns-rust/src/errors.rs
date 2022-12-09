use std::error::Error;
use std::ffi;
use std::fmt;

use anyhow::anyhow;

#[derive(Debug, thiserror::Error)]
pub enum CGNSError {
    #[error(transparent)]
    CGNSLibError(#[from] CGNSLibraryError),
    // there is work to do here...
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<ffi::FromBytesWithNulError> for CGNSError {
    fn from(err: ffi::FromBytesWithNulError) -> Self {
        Self::Other(anyhow!(
            "Could not convert str from bytes: {}",
            err.to_string()
        ))
    }
}

impl From<ffi::NulError> for CGNSError {
    fn from(err: ffi::NulError) -> Self {
        Self::Other(anyhow!(
            "Could not convert str from bytes: {}",
            err.to_string()
        ))
    }
}

impl From<std::str::Utf8Error> for CGNSError {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::Other(anyhow!(
            "Could not convert str from bytes: {}",
            err.to_string()
        ))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FFIStringError {
    Null(#[from] ffi::NulError),
    IntoString(#[from] ffi::IntoStringError),
    FromVec(#[from] ffi::FromVecWithNulError),
    FromBytes(#[from] ffi::FromBytesWithNulError),
}

impl fmt::Display for FFIStringError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(fmt, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub struct CGNSLibraryError(pub *const ffi::c_char);

impl fmt::Display for CGNSLibraryError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        if self.0.is_null() {
            unreachable!("cg_get_error returned a null pointer");
        }
        let msg = unsafe { ffi::CStr::from_ptr(self.0) }
            .to_str()
            .unwrap_or("(could not read CGNS error)");
        write!(fmt, "CGNS: {}", msg)
    }
}

impl Error for CGNSLibraryError {}

unsafe impl Send for CGNSLibraryError {}
unsafe impl Sync for CGNSLibraryError {}
