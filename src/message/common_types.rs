use crate::message;
use crate::message::ToMessage;
use prost::Message;

pub mod common_types {
    include!(concat!(env!("OUT_DIR"), "/message.common_types.rs"));
}
use common_types::*;

macro_rules! impl_to_message_for_primitive {
    ($t:ty, $wrapper:ident) => {
        impl_to_message_for_primitive!($t, $wrapper, (*self), String);
    };
    ($t:ty, $wrapper:ident, $converter:expr) => {
        impl_to_message_for_primitive!($t, $wrapper, $converter(*self), String);
    };
    ($t:ty, $wrapper:ident, $conversion:tt, $noop_:tt) => {
        impl ToMessage for $t {
            fn to_message(&self) -> Box<dyn prost::Message> {
                Box::new($wrapper { value: $conversion })
            }
            fn is_primitive<L: message::private::IsLocal>(&self) -> bool {
                return true;
            }
        }
    };
}

// TODO: This doesn't work. I want to define a simple macro for implementing all of these
//       ToMessage impls. But the proto types don't line up exactly. So sometimes these impls
//       need to have a conversion function. Sometimes they fit and don't. So I just need to
//       get this macro to work with or without the conversion function.
impl_to_message_for_primitive!(u8, U32Wrapper, u32::from);
