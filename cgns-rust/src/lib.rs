//! `rust-cgns` is a friendly but low-level wrapper around `cgns-sys`.
//!
//! Start by looking at [`file::File`], which implements [`traits::CGNSParent`] for [`file::base::Base`].

mod errors;
pub mod file;
pub mod library;
pub mod traits;
mod utils;

pub use cgns_sys;
