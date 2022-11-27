pub mod base;

use std::ffi;
use std::os::unix::prelude::OsStrExt;
use std::path::Path;

use anyhow::{anyhow, Result};
use cgns_sys::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{
    traits::{CGNSNode, CGNSParent},
    utils::{ier_cg_fn, CGNSError},
    Library,
};

use self::base::Base;

#[derive(Debug, Clone, PartialEq)]
pub struct File {
    /// File ID
    desc: ffi::c_int,
    /// File CGNS version
    pub version: f32,
}

#[repr(u32)]
#[derive(Debug, Clone, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum OpenFileMode {
    Read = CG_MODE_READ,
    Write = CG_MODE_WRITE,
    Modify = CG_MODE_MODIFY,
}

impl File {
    pub fn new<P>(path: P, mode: OpenFileMode) -> Result<Self>
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
            desc: cg_fn,
            version,
        })
    }

    unsafe fn close_by_ref(&self) -> Result<()> {
        ier_cg_fn!(cg_close(self.desc))
    }

    pub fn close(self) -> Result<()> {
        unsafe { self.close_by_ref()? };
        std::mem::forget(self);
        Ok(())
    }
}

impl Drop for File {
    fn drop(&mut self) {
        let res = unsafe { self.close_by_ref() };
        if let Err(e) = res {
            panic!("Failed to close {:?}: {}", self, e)
        }
    }
}

impl<'a> CGNSNode<'a> for File {
    type Parent = Library;
    fn id(&self) -> i32 {
        self.desc
    }
    fn parent(&self) -> &Self::Parent {
        panic!();
    }
    fn from_id(_parent: &'a Self::Parent, _id: i32) -> Result<Self> {
        panic!();
    }
}

impl<'a> CGNSParent<'a, Base<'a>> for File {
    fn num_child(&self) -> Result<i32> {
        let mut number = 0;
        ier_cg_fn!(cg_nbases(self.id(), &mut number))?;
        Ok(number)
    }
}
