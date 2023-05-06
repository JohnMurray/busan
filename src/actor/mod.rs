//! Actor trait and related types necessary for basic actor functionality.
//!
//! The [`actor`](crate::actor) module contains the necessary core components when writing
//! an `Actor` implementation and deals with construction, message sending and receiving, and
//! various lifecycle hooks.
//!
//! ## Actor Construction
//!
//! Construction of an actor looks different than how types are normally constructed in Rust. This
//! is because actors, in order to satisfy certain properties of the actor model, must be isolated.
//! That is, no state can be shared across the boundary of an actor. To satisfy this, construction
//! looks like the following:
//!
//!   + The constructor is defined within the [`ActorInit`] trait
//!   + Construction is defined as taking a `[`Message`](crate::message::Message) and returning
//!     an [`Actor`] instance
//!   + Construction is performed indirectly through either
//!     [`ActorSystem::spawn_root_actor`](crate::ActorSystem::spawn_root_actor) or
//!     [`Context::spawn_child`]
//!
//! With this construction, isolation is satisfied by:
//!   + Instances of actors are only available internally to the actor system while user code
//!     interacts with actors through [`ActorAddress`] handles.
//!   + Construction properties can be serialized which ensures no internal state is accidentally
//!     shared
//!
//! ## Actor Implementation
//!
//! Actors in busan are currently very simple. They can receive messages and, using the context,
//! send messages or spawn other actors. There are currently no limitations on the type of internal
//! state an actor can have other `Actor`s must be `Send`'able (which allows for scheduling across
//! executors).
//!
//! ## Example
//!
//! ```rust
//! use busan::message::common_types::StringWrapper;
//! use busan::prelude::*;
//!
//! #[derive(Clone, PartialEq, prost::Message, busan::Message)]
//! struct GreeterInit {
//!   #[prost(string, tag = "1")]
//!   phrase: String,
//! }
//!
//! struct Greeter {
//!   phrase: String,
//!   greet_count: u32,
//! }
//!
//! impl ActorInit for Greeter {
//!   type Init = GreeterInit;
//!   fn init(init_msg: &Self::Init) -> Self {
//!     let mut phrase = init_msg.phrase.clone();
//!     if phrase.is_empty() {
//!       phrase = "Hello".to_string();
//!     }
//!     Self { phrase, greet_count: 0 }
//!   }
//! }
//!
//! impl Actor for Greeter {
//!   fn receive(&mut self, ctx: Context, msg: Box<dyn Message>) {
//!     if let Some(greet_msg) = msg.as_any().downcast_ref::<StringWrapper>() {
//!       println!("{} {} (greetings: {})", self.phrase, greet_msg.value, self.greet_count);
//!       self.greet_count += 1;
//!     }
//!   }
//! }
//! ```
//! The example uses a protobuf message type for demonstration, but there are a couple of things to
//! note:
//!   + Protobuf types can be generated using a build-script
//!   + Basic type-wrappers already exist for common values, so we could have used
//!     [`StringWrapper`](crate::message::common_types::StringWrapper) instead of defining our own
//!     protobuf message
//!
//! <!-- TODO: Link to the 'patterns' module when it exists -->

// Allow this since we're re-exporting everything and just re-using the module name for
// our own organization of files.
#![allow(clippy::module_inception)]

// Hide all of the sub-modules. These are broken out for easier organization of the code, but
// conceptually these all belong in the same module and are re-exported here.
#[doc(hidden)]
pub mod actor;
#[doc(hidden)]
pub mod address;
#[doc(hidden)]
pub mod letter;

#[doc(inline)]
pub use actor::*;
#[doc(inline)]
pub use address::*;

pub(crate) use letter::*;
pub(crate) type Mailbox = crossbeam_channel::Sender<Letter>;
