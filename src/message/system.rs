pub mod proto {
    use crate::message::common_types::impl_busan_message;
    use crate::message::Message;

    include!(concat!(env!("OUT_DIR"), "/message.system.rs"));
    impl_busan_message!(Ack);
}

pub use proto::Ack;

/// Create an ACK message given a nonce.
pub fn ack(nonce: u32) -> proto::Ack {
    proto::Ack { nonce }
}
