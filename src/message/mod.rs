pub mod common_types;

pub trait Message: prost::Message {
    fn as_any(&self) -> &dyn std::any::Any;

    fn encode_to_vec2(&self) -> Vec<u8>;

    #[doc(hidden)]
    fn encoded_len(&self) -> usize {
        prost::Message::encoded_len(self)
    }

    // fn decode_from_vec(bytes: Vec<u8>) -> Result<Self, prost::DecodeError>
    // where
    //     Self: Sized,
    // {
    //     prost::Message::decode(bytes)
    // }
}

pub trait ToMessage<M: Message> {
    fn to_message(self) -> M;

    fn is_primitive<L: private::IsLocal>(&self) -> bool {
        false
    }
}

/// Impl ToMessage for all types that are already messages.
impl<M: Message> ToMessage<M> for M {
    fn to_message(self) -> M {
        self
    }
}

/*
 * Use a private module to create a private trait so we can use this on methods in
 * ToMessage so that they can _only_ be implemented and called within our crate.
 */
pub(crate) mod private {
    #[doc(hidden)]
    pub enum Local {}
    #[doc(hidden)]
    pub trait IsLocal {}
    impl IsLocal for Local {}
}
