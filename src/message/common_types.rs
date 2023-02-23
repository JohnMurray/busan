use crate::message;
use crate::message::ToMessage;

pub mod common_types {
    include!(concat!(env!("OUT_DIR"), "/message.common_types.rs"));
}
use common_types::*;

macro_rules! impl_to_message_for_primitive {
    ($t:ty, $wrapper:ident) => {
        impl_to_message_for_primitive!($t, $wrapper, |x| x);
    };
    ($t:ty, $wrapper:ident, $converter:expr $(, $deref:tt)?) => {
        impl ToMessage for $t {
            fn to_message(self) -> Box<dyn prost::Message> {
                Box::new($wrapper {
                    value: $converter($($deref)* self),
                })
            }
            fn is_primitive<L: message::private::IsLocal>(&self) -> bool {
                return true;
            }
        }
    };
}

impl_to_message_for_primitive!(u8, U32Wrapper, u32::from);
impl_to_message_for_primitive!(&u8, U32Wrapper, u32::from, *);
impl_to_message_for_primitive!(u16, U32Wrapper, u32::from);
impl_to_message_for_primitive!(&u16, U32Wrapper, u32::from, *);
impl_to_message_for_primitive!(u32, U32Wrapper);
impl_to_message_for_primitive!(&u32, U32Wrapper, |x| x, *);
impl_to_message_for_primitive!(u64, U64Wrapper);
impl_to_message_for_primitive!(&u64, U64Wrapper, |x| x, *);
impl_to_message_for_primitive!(i8, I32Wrapper, i32::from);
impl_to_message_for_primitive!(&i8, I32Wrapper, i32::from, *);
impl_to_message_for_primitive!(i16, I32Wrapper, i32::from);
impl_to_message_for_primitive!(&i16, I32Wrapper, i32::from, *);
impl_to_message_for_primitive!(i32, I32Wrapper);
impl_to_message_for_primitive!(&i32, I32Wrapper, |x| x, *);
impl_to_message_for_primitive!(i64, I64Wrapper);
impl_to_message_for_primitive!(&i64, I64Wrapper, |x| x, *);
impl_to_message_for_primitive!(f32, FloatWrapper);
impl_to_message_for_primitive!(&f32, FloatWrapper, |x| x, *);
impl_to_message_for_primitive!(f64, DoubleWrapper);
impl_to_message_for_primitive!(&f64, DoubleWrapper, |x| x, *);
impl_to_message_for_primitive!(bool, BoolWrapper);
impl_to_message_for_primitive!(&bool, BoolWrapper, |x| x, *);
impl_to_message_for_primitive!(String, StringWrapper);
impl_to_message_for_primitive!(&String, StringWrapper, |x: &String| x.clone());
impl_to_message_for_primitive!(&str, StringWrapper, |x: &str| x.to_string());
