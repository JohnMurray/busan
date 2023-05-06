//! Busan (부산) is an actor framework that prioritizes flexible construction and organization
//! of large-scale systems with a focus on ergonomics and usability.
//!
//! Busan is currently under initial development and may not be suitable for serious use-cases
//! (yet).
//!
//! <!-- TODO: Write a simple example here -->
//! <!-- TODO: Include a link to the example projects -->

#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]

pub mod actor;
pub mod config;
pub mod executor;
pub mod message;
pub mod prelude;
pub mod util;

#[doc(hidden)]
pub mod system;

#[doc(inline)]
pub use system::ActorSystem;

#[allow(unused_imports)]
#[macro_use]
extern crate busan_derive;
#[doc(hidden)]
pub use busan_derive::Message;
