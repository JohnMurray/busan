use crate::actor::ActorAddress;
use crate::message::Message;
use std::fmt::{Display, Formatter};

/// `Envelope` is the internal representation for messages sent between actors, containing the
/// actual message (the payload) as well as some additional meta-data (currently just the sender).
#[derive(Debug)]
pub(crate) struct Envelope {
    pub(crate) sender: SenderType,
    pub(crate) payload: Box<dyn Message>,
}

/// `SenderType` to abstractly represent the sender on a Envelope. While the sender _could_ simply
/// be represented as an `ActorAddress` always, this might add a lot of additional overhead that
/// is otherwise not necessary. For example, an actor sending a message to itself will not need
/// the address. Similarly, other circumstances may not require transmission of the address.
#[derive(Debug)]
pub(crate) enum SenderType {
    Actor(ActorAddress),

    /// A message that originates from the system will not have a sender address and is a
    /// bit of an exception to the rule.
    System,

    /// A message that is sent by the parent (and thus is being received by the child) does not
    /// need to have the address as all children should have a reference to their parent upon
    /// creation.
    Parent,

    /// An actor sent a message to themselves (e.g. deferred processing, loopback, startup message,
    /// etc.) Obviously we do not need to transmit the address in this case.
    SentToSelf,
}

impl Display for SenderType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SenderType::Actor(addr) => addr.fmt(f),
            SenderType::System => write!(f, "System"),
            SenderType::Parent => write!(f, "Parent"),
            SenderType::SentToSelf => write!(f, "Self"),
        }
    }
}

impl Envelope {
    /// Construct a new envelope. This will automatically determine the sender type based on the
    /// sender and receiver addresses. A `None` sender will always be interpreted as a
    /// `SenderType::System`.
    pub fn new(
        sender: Option<ActorAddress>,
        receiver: &ActorAddress,
        payload: Box<dyn Message>,
    ) -> Self {
        match sender {
            None => Self {
                sender: SenderType::System,
                payload,
            },
            Some(sender) => {
                if sender.uri == receiver.uri {
                    Self {
                        sender: SenderType::SentToSelf,
                        payload,
                    }
                } else if receiver.is_parent(&sender) {
                    // The sender is the parent to the receiver
                    Self {
                        sender: SenderType::Parent,
                        payload,
                    }
                } else {
                    Self {
                        sender: SenderType::Actor(sender),
                        payload,
                    }
                }
            }
        }
    }
}
