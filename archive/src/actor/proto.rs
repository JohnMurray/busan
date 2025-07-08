//! Serializable message types for the actor module
//!
//! This module contains the protobuf definitions that map to internal types
//! such as [`ActorAddress`](crate::actor::ActorAddress) as well as the associated
//! [`ToMessage`](crate::message::ToMessage) implementations for these types.

use crate::actor;
use crate::message::common_types::impl_busan_message;
use crate::message::{Message, ToMessage};
use std::cell::RefCell;

// Import the generated protobuf definitions (see build.rs)
include!(concat!(env!("OUT_DIR"), "/actor.proto.rs"));

impl_busan_message!(ActorAddress);
impl_busan_message!(AddressList);

impl ToMessage<ActorAddress> for &actor::ActorAddress {
    fn to_message(self) -> ActorAddress {
        ActorAddress {
            scheme: match self.uri.scheme {
                Scheme::Local => Scheme::Local as i32,
                Scheme::Remote => Scheme::Remote as i32,
            },
            path: self.uri.path(),
        }
    }
}

impl TryFrom<ActorAddress> for actor::ActorAddress {
    type Error = String;

    fn try_from(address: ActorAddress) -> Result<Self, Self::Error> {
        let scheme = Scheme::from_i32(address.scheme)
            .ok_or(format!("Invalid scheme: {}", address.scheme))?;
        Ok(actor::ActorAddress {
            // TODO: This needs an internal constructor. Presumably there is at least
            //       one other place we're doing this same thing
            uri: actor::Uri {
                scheme,
                path_segments: address.path.split('/').map(|s| s.to_string()).collect(),
            },
            mailbox: RefCell::new(None),
        })
    }
}

impl ToMessage<AddressList> for &[actor::ActorAddress] {
    fn to_message(self) -> AddressList {
        AddressList {
            addresses: self.iter().map(|a| a.to_message()).collect(),
        }
    }
}
