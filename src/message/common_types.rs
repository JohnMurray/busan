use crate::message;
use crate::message::{Message, ToMessage};

include!(concat!(env!("OUT_DIR"), "/message.common_types.rs"));

macro_rules! impl_to_message_for_primitive {
    ($t:ty, $wrapper:ident) => {
        impl_to_message_for_primitive!($t, $wrapper, |x| x);
    };
    ($t:ty, $wrapper:ident, $converter:expr $(, $deref:tt)?) => {
        impl ToMessage for $t {
            fn to_message(self) -> Box<dyn Message> {
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

macro_rules! impl_to_message_for_primitive_list {
    // Owned types that don't need conversion
    ($t:ty, $wrapper:ident) => {
        impl ToMessage for Vec<$t> {
            fn to_message(self) -> Box<dyn Message> {
                Box::new($wrapper { values: self })
            }
        }
    };
    // Owned types that need conversion
    ($t:ty, $wrapper:ident, $converter:expr) => {
        impl ToMessage for Vec<$t> {
            fn to_message(self) -> Box<dyn Message> {
                Box::new($wrapper {
                    values: self.iter().map(|x| $converter(*x)).collect(),
                })
            }
        }
    };
    // Borrowed types that don't need conversion
    (&$t:ty, $wrapper:ident, clone) => {
        impl ToMessage for Vec<$t> {
            fn to_message(self) -> Box<dyn Message> {
                Box::new($wrapper {
                    values: self.clone(),
                })
            }
        }
    };
    // Borrowed types that need conversion
    (&$t:ty, $wrapper:ident, $converter:expr) => {
        impl ToMessage for Vec<$t> {
            fn to_message(self) -> Box<dyn Message> {
                Box::new($wrapper {
                    values: self.iter().map(|x| $converter(x)).collect(),
                })
            }
        }
    };
}

impl_to_message_for_primitive_list!(u8, U32ListWrapper, u32::from);
impl_to_message_for_primitive_list!(u16, U32ListWrapper, u32::from);
impl_to_message_for_primitive_list!(u32, U32ListWrapper);
impl_to_message_for_primitive_list!(u64, U64ListWrapper);
impl_to_message_for_primitive_list!(i8, I32ListWrapper, i32::from);
impl_to_message_for_primitive_list!(i16, I32ListWrapper, i32::from);
impl_to_message_for_primitive_list!(i32, I32ListWrapper);
impl_to_message_for_primitive_list!(i64, I64ListWrapper);
impl_to_message_for_primitive_list!(f32, FloatListWrapper);
impl_to_message_for_primitive_list!(f64, DoubleListWrapper);
impl_to_message_for_primitive_list!(bool, BoolListWrapper);
impl_to_message_for_primitive_list!(String, StringListWrapper);
impl_to_message_for_primitive_list!(&String, StringListWrapper, |x: &String| x.clone());
impl_to_message_for_primitive_list!(&str, StringListWrapper, |x: &str| x.to_string());

macro_rules! impl_busan_message {
    ($t:ty) => {
        impl Message for $t {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    };
}

impl_busan_message!(U32Wrapper);
impl_busan_message!(U64Wrapper);
impl_busan_message!(I32Wrapper);
impl_busan_message!(I64Wrapper);
impl_busan_message!(FloatWrapper);
impl_busan_message!(DoubleWrapper);
impl_busan_message!(BoolWrapper);
impl_busan_message!(StringWrapper);
impl_busan_message!(U32ListWrapper);
impl_busan_message!(U64ListWrapper);
impl_busan_message!(I32ListWrapper);
impl_busan_message!(I64ListWrapper);
impl_busan_message!(FloatListWrapper);
impl_busan_message!(DoubleListWrapper);
impl_busan_message!(BoolListWrapper);
impl_busan_message!(StringListWrapper);
