//! Based on: <https://cgns.github.io/CGNS_docs_current/midlevel/fileops.html>

pub mod base;

use std::ffi;
use std::marker::PhantomData;
use std::os::unix::prelude::OsStrExt;
use std::path::Path;

use cgns_sys::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use self::base::Base;
use crate::library::LibraryHandle;
use crate::traits::{CGNSNode, CGNSNodeIterator, CGNSParent};
use crate::utils::{ier_cg_fn, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct File<'l> {
    /// File ID
    id: ffi::c_int,
    /// File CGNS version
    pub version: f32,
    _library: PhantomData<&'l ()>,
}

#[repr(u32)]
#[derive(Debug, Clone, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum OpenFileMode {
    Read = CG_MODE_READ,
    Write = CG_MODE_WRITE,
    Modify = CG_MODE_MODIFY,
}

impl<'l> File<'l> {
    pub fn new<P>(_lib: &'l LibraryHandle, path: P, mode: OpenFileMode) -> Result<Self>
    where
        P: AsRef<Path> + Sized,
    {
        let path = path.as_ref();
        let mut cg_fn = 0;
        let mut version = 0.;
        let raw_path = ffi::CString::new(path.as_os_str().as_bytes()).unwrap();
        let mode: u32 = mode.into();
        ier_cg_fn!(cg_open(raw_path.as_ptr(), mode as i32, &mut cg_fn,))?;
        ier_cg_fn!(cg_version(cg_fn, &mut version))?;

        Ok(Self {
            id: cg_fn,
            version,
            _library: PhantomData,
        })
    }

    /// Save the CGNS file.
    /// `copy_links` determines whether links are left intact or replaced by a copy of the associated data in the new file.
    pub fn save_as<P: AsRef<Path>>(&self, path: P, copy_links: bool) -> Result {
        let path = ffi::CString::new(path.as_ref().as_os_str().as_bytes())?;
        ier_cg_fn!(cg_save_as(
            self.id,
            path.as_ptr(),
            CG_FILE_HDF5 as i32,
            copy_links as i32
        ))
    }

    /// This closes self without consuming it. Necessary to close the file in [`Drop`]
    #[allow(unused_unsafe)]
    unsafe fn close_by_ref(&self) -> Result<()> {
        ier_cg_fn!(cg_close(self.id))
    }

    pub fn close(self) -> Result<()> {
        unsafe { self.close_by_ref()? };
        std::mem::forget(self);
        Ok(())
    }

    pub fn iter_bases(&self) -> Result<CGNSNodeIterator<Base>> {
        self.iter()
    }
}

impl<'l> Drop for File<'l> {
    fn drop(&mut self) {
        let res = unsafe { self.close_by_ref() };
        if let Err(e) = res {
            panic!("Failed to close {:?}: {}", self, e)
        }
    }
}

impl<'l> CGNSNode<'l> for File<'l> {
    type Parent = LibraryHandle;
    fn id(&self) -> i32 {
        self.id
    }
    fn parent(&self) -> &Self::Parent {
        panic!();
    }
    fn from_id(_parent: &'l Self::Parent, _id: i32) -> Result<Self> {
        panic!();
    }
}

impl<'l> CGNSParent<'l, Base<'l>> for File<'l> {
    fn num_child(&self) -> Result<i32> {
        let mut number = 0;
        ier_cg_fn!(cg_nbases(self.id(), &mut number))?;
        Ok(number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use testdir::testdir;

    macro_rules! fn_name {
        () => {{
            fn f() {}
            fn type_name_of<T>(_: T) -> &'static str {
                std::any::type_name::<T>()
            }
            let name = type_name_of(f);
            &name[..name.len() - 3]
        }};
    }

    /// returns a CGNS file in [`OpenFileMode::Modify`] mode.
    pub fn cgns_file(library: &LibraryHandle, dir: PathBuf, id: u32) -> (PathBuf, File) {
        let file_name = format!("{}-{}.cgns", fn_name!(), id);
        let path = dir.join(file_name);
        let file = library.open(path.clone(), OpenFileMode::Write).unwrap();
        file.close().unwrap();
        let file = library.open(path.clone(), OpenFileMode::Modify).unwrap();
        (path, file)
    }

    #[test]
    fn can_write_cgns_file() {
        let library = LibraryHandle::acquire();
        let (p, f) = cgns_file(&library, testdir!(), 0);
        f.close().unwrap();
        assert!(p.is_file());
    }

    #[test]
    fn can_open_cgns_file() {
        let library = LibraryHandle::acquire();
        let (p, f) = cgns_file(&library, testdir!(), 1);
        f.close().unwrap();
        assert!(p.is_file());
        let f = library.open(p, OpenFileMode::Read).unwrap();
        assert_eq!(f.num_child().unwrap(), 0);
        f.close().unwrap();
    }
}
