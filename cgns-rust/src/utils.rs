use std::ffi::{self, CString};
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

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

pub(crate) fn string2bytes(str: &str) -> Result<CString> {
    let str = format!("{str}\0");
    let bytes = str.into_bytes();
    let cstr = CString::from_vec_with_nul(bytes)?;
    Ok(cstr)
}

/// Convert a path to the narrow string accepted by CGNS without lossy replacement.
pub(crate) fn path2bytes(path: &Path) -> Result<CString> {
    // Unix paths are arbitrary bytes; keep non-UTF-8 filenames intact.
    #[cfg(unix)]
    let bytes = path.as_os_str().as_bytes();

    // CGNS accepts char*, not Windows UTF-16 strings.
    #[cfg(not(unix))]
    let bytes = path
        .to_str()
        .ok_or_else(|| CGNSError::InvalidFileError("Path cannot be represented as UTF-8".into()))?
        .as_bytes();

    Ok(CString::new(bytes)?)
}

/// Equivalent to `copy_from_slice()` without the requirement `src.len() == dst.len()`.
#[inline]
pub(crate) fn copy_from_mismatched_slice<T: Copy>(dst: &mut [T], src: &[T]) {
    let len = dst.len().min(src.len());
    dst[..len].copy_from_slice(&src[..len])
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::FFIError;

    #[test]
    fn path_conversion_preserves_utf8() {
        for name in [
            "mesh.cgns",
            "dir with spaces/mesh.cgns",
            "maillage-é-🦀.cgns",
        ] {
            assert_eq!(
                path2bytes(Path::new(name)).unwrap().to_bytes(),
                name.as_bytes()
            );
        }
    }

    #[test]
    fn path_conversion_rejects_nul() {
        assert!(matches!(
            path2bytes(Path::new("mesh\0.cgns")),
            Err(CGNSError::FFIError(FFIError::Null(_)))
        ));
    }

    #[cfg(unix)]
    #[test]
    fn path_conversion_preserves_non_utf8_unix_paths() {
        let bytes = b"mesh-\xff.cgns";
        let path = Path::new(ffi::OsStr::from_bytes(bytes));
        assert_eq!(path2bytes(path).unwrap().to_bytes(), bytes);
    }

    #[cfg(windows)]
    #[test]
    fn path_conversion_rejects_unpaired_surrogates() {
        use std::os::windows::ffi::OsStringExt;

        let name = ffi::OsString::from_wide(&[0xd800]);
        assert!(matches!(
            path2bytes(Path::new(&name)),
            Err(CGNSError::InvalidFileError(_))
        ));
    }
}
