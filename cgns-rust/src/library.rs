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
pub struct LibraryHandle {
    _phantom: PhantomData<*const ()>,
}

impl LibraryHandle {
    /// Try to acquire a unique instance of [`LibraryHandle`].
    /// Returns [`Err`] if another thread has already aquired a handle.
    pub fn try_acquire() -> Result<Self, &'static str> {
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

    /// Acquires a unique instance of [`LibraryHandle`] by spin-locking.
    pub fn acquire() -> Self {
        let base_sleep_dur = std::time::Duration::from_millis(10);
        let mut sleep_multiplier = 0;

        while LIB_IN_USE
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
            .is_err()
        {
            std::thread::sleep(base_sleep_dur * sleep_multiplier);
            sleep_multiplier = (sleep_multiplier + 1).min(500);
        }

        Self {
            _phantom: Default::default(),
        }
    }

    /// # Safety
    /// None. This is as dangerous as reading a mutex's contents without locking it.
    pub unsafe fn fake_aquire() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }

    /// Open a [`File`].
    pub fn open<P>(&self, path: P, mode: OpenFileMode) -> Result<File>
    where
        P: AsRef<Path> + Sized,
    {
        File::new(self, path, mode)
    }
}

impl Drop for LibraryHandle {
    fn drop(&mut self) {
        if LIB_IN_USE
            .compare_exchange(true, false, Ordering::Release, Ordering::Relaxed)
            .is_err()
        {
            unreachable!("Singleton state mismatch");
        }
    }
}

impl std::fmt::Debug for LibraryHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "CGNSLibHandle")
    }
}
