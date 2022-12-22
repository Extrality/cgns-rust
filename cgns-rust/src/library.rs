//! Represents an instance of the CGNS library

pub use cgns_sys;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use super::file::{File, OpenFileMode};
use super::utils::Result;

/// Assures at runtime that the Library struct is not instanciated twice
pub static LIB_IN_USE: AtomicBool = AtomicBool::new(false);

/// represents access to the CGNS library. Only one instance can exist at a time
/// due to the design of the CGNS library
pub struct Library {
    _phantom: PhantomData<*const ()>,
}

impl Library {
    pub fn new() -> Result<Self, &'static str> {
        Self::take()
    }
    pub fn take() -> Result<Self, &'static str> {
        if LIB_IN_USE
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
            .is_err()
        {
            Err("The CGNS library is already in use.")
        } else {
            Ok(Self {
                _phantom: Default::default(),
            })
        }
    }

    pub fn open<P>(&self, path: P, mode: OpenFileMode) -> Result<File>
    where
        P: AsRef<Path> + Sized,
    {
        File::new(self, path, mode)
    }
}
impl Drop for Library {
    fn drop(&mut self) {
        if LIB_IN_USE
            .compare_exchange(true, false, Ordering::Release, Ordering::Relaxed)
            .is_err()
        {
            unreachable!("Singleton state mismatch");
        }
    }
}
impl std::fmt::Debug for Library {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "CGNSLib")
    }
}
