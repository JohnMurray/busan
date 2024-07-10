//! Core message types used by Busan and primitive type wrappers

pub mod common_types;
pub mod system;

pub trait Message: prost::Message {
    fn as_any(&self) -> &dyn std::any::Any;

    /// A version of encode_to_vec that does not have a default implementation or
    /// `Self: Sized` requirement. This allows us to implement this directly on the
    /// type and use dynamic dispatch indirectly call `encode_to_vec` on `prost::Message`
    /// and satisfy the `Sized` requirement.
    #[doc(hidden)]
    fn encode_to_vec2(&self) -> Vec<u8>;

    /// A version of merge that does not have a [`Sized`] requirement
    #[doc(hidden)]
    fn merge2(&mut self, buf: &[u8]) -> Result<(), prost::DecodeError>;

    #[doc(hidden)]
    fn encoded_len(&self) -> usize {
        prost::Message::encoded_len(self)
    }
}

/// A simple trait for converting types to a Message that can be sent/received
/// to/from an actor. Many of the entrypoints to sending messages will use
/// `ToMessage` rather than `Message`, directly.
pub trait ToMessage<M: Message> {
    fn to_message(self) -> M;

    /// Signal that implementation is for a primitive type (e.g. ints, floats,
    /// bools, etc). This may allow for some internal optimizations.
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
    pub trait IsLocal {}
    #[doc(hidden)]
    pub enum Local {}
    impl IsLocal for Local {}
}
