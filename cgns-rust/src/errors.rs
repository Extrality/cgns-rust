use std::error::Error;
use std::ffi;
use std::fmt;

#[derive(Debug, thiserror::Error)]
pub enum CGNSError {
    /// Errors coming from the CGNS MLL
    #[error(transparent)]
    CGNSLibError(#[from] CGNSLibraryError),
    /// Error during FFI type conversion
    #[error("Error in FFI interface: {0}")]
    FFIError(FFIError),
    /// Catch-all error type (to phase out)
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Allows building a CGNSError from any FFIError member
impl<E> From<E> for CGNSError
where
    FFIError: From<E>,
{
    fn from(err: E) -> Self {
        CGNSError::FFIError(err.into())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FFIError {
    #[error(transparent)]
    Null(#[from] ffi::NulError),
    #[error(transparent)]
    IntoString(#[from] ffi::IntoStringError),
    #[error(transparent)]
    FromVec(#[from] ffi::FromVecWithNulError),
    #[error(transparent)]
    FromBytes(#[from] ffi::FromBytesWithNulError),
    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),
}

#[derive(Clone)]
pub struct CGNSLibraryError(pub *const ffi::c_char);

impl CGNSLibraryError {
    fn msg(&self) -> &'static str {
        if self.0.is_null() {
            unreachable!("cg_get_error returned a null pointer");
        }
        unsafe { ffi::CStr::from_ptr(self.0) }
            .to_str()
            .unwrap_or("could not read CGNS error")
    }
}

impl fmt::Debug for CGNSLibraryError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(fmt, "CGNSLibraryError({})", self.msg())
    }
}

impl fmt::Display for CGNSLibraryError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(fmt, "CGNSLibraryError: {}", self.msg())
    }
}

impl Error for CGNSLibraryError {}
unsafe impl Send for CGNSLibraryError {}
unsafe impl Sync for CGNSLibraryError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ffi_error_displays_correctly() {
        let result = ffi::CStr::from_bytes_with_nul(&[b'a', 0, b'b']);
        let ffi_error = result.unwrap_err();
        let ffi_error_display = format!("{}", ffi_error);
        let ffi_error_debug = format!("{:?}", ffi_error);
        // Check Display on FFIError looks OK
        assert_eq!(
            ffi_error_display,
            "data provided contains an interior nul byte at byte pos 1"
        );
        let error: CGNSError = ffi_error.into();
        let error_display = format!("{}", error);
        let error_debug = format!("{:?}", error);
        // Check CGNSError is correctly displaying FFIError
        assert_eq!(
            error_display,
            format!("Error in FFI interface: {}", ffi_error_display)
        );
        assert_eq!(
            error_debug,
            format!("FFIError(FromBytes({}))", ffi_error_debug)
        );
    }
}
