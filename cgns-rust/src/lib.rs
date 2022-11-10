//! `rust-cgns` is a friendly but low-level wrapper around `cgns-sys`.
//!
//! Start by looking at [`file::File`], which implements [`traits::CGNSParent`] for [`file::base::Base`].

mod utils;

pub mod file;
pub mod traits;

/// TODO: Force users to instanciate [`Library`] to lock CGNS to a single thread
/// (pcgns is out of scope).
pub struct Library {} // TODO
