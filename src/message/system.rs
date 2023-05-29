//! System messages that have special properties

use prost::DecodeError;
use std::any::Any;

#[derive(prost::Message)]
pub struct PoisonPill {}

impl super::Message for PoisonPill {
    fn as_any(&self) -> &dyn Any {
        return self;
    }

    fn encode_to_vec2(&self) -> Vec<u8> {
        prost::Message::encode_to_vec(self)
    }

    fn merge2(&mut self, buf: &[u8]) -> Result<(), DecodeError> {
        prost::Message::merge(self, buf)
    }

    fn is_system_message(&self) -> bool {
        true
    }
}
