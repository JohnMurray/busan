//! Serializable message types for the actor module
//!
//! This module contains the protobuf definitions that map to internal types
//! such as [`ActorAddress`](crate::actor::ActorAddress) as well as the associated
//! [`ToMessage`](crate::message::ToMessage) implementations for these types.

use crate::actor;
use crate::actor::UriScheme;
use crate::message::common_types::impl_busan_message;
use crate::message::{Message, ToMessage};
use std::cell::RefCell;

// Import the generated protobuf definitions (see build.rs)
include!(concat!(env!("OUT_DIR"), "/actor.address.rs"));

impl_busan_message!(ActorAddress);
impl_busan_message!(AddressList);

impl ToMessage<ActorAddress> for &actor::ActorAddress {
    fn to_message(self) -> ActorAddress {
        ActorAddress {
            scheme: match self.uri.scheme {
                UriScheme::Local => Scheme::Local as i32,
                UriScheme::Remote => Scheme::Remote as i32,
            },
            path: self.uri.path(),
        }
    }
}

impl From<ActorAddress> for actor::ActorAddress {
    fn from(address: ActorAddress) -> Self {
        let scheme = match address.scheme {
            s if s == Scheme::Local as i32 => UriScheme::Local,
            s if s == Scheme::Remote as i32 => UriScheme::Remote,
            // TODO: Should this be implemented as a TryFrom instead?
            _ => panic!("Invalid scheme"),
        };
        actor::ActorAddress {
            // TODO: This needs an internal constructor. Presumably there is at least
            //       one other place we're doing this same thing
            uri: actor::Uri {
                scheme,
                path_segments: address.path.split('/').map(|s| s.to_string()).collect(),
            },
            mailbox: RefCell::new(None),
        }
    }
}

impl ToMessage<AddressList> for &[actor::ActorAddress] {
    fn to_message(self) -> AddressList {
        AddressList {
            addresses: self.iter().map(|a| a.to_message()).collect(),
        }
    }
}
