// Allow this since we're re-exporting everything and just re-using the module name for
// our own organization of files.
#![allow(clippy::module_inception)]

pub mod actor;
pub mod address;
pub mod letter;

pub use actor::*;
pub use address::*;
pub use letter::*;

pub(crate) type Mailbox = crossbeam_channel::Sender<Letter>;
