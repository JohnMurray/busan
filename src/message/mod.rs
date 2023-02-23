pub mod common_types;

pub trait ToMessage {
    fn to_message(self) -> Box<dyn prost::Message>;

    fn is_primitive<L: private::IsLocal>(&self) -> bool {
        return false;
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
