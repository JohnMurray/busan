use crossbeam_channel::{Receiver, Sender};

/// A simple wrapper around channels to abstract out the specific channel implementation
/// being used for sending and receiving. This ideally should be the only module that
/// import anything from crossbeam_channel.
pub struct CommandChannel<T> {
    pub(self) sender: Sender<T>,
    pub(self) receiver: Receiver<T>,
}

impl<T> CommandChannel<T> {
    pub fn new() -> CommandChannel<T> {
        let (sender, receiver) = crossbeam_channel::unbounded();
        CommandChannel { sender, receiver }
    }

    pub fn send(&self, command: T) -> Result<(), crossbeam_channel::SendError<T>> {
        self.sender.send(command)
    }

    pub fn recv(&self) -> Result<T, crossbeam_channel::RecvError> {
        self.receiver.recv()
    }

    pub fn recv_is_empty(&self) -> bool {
        self.receiver.is_empty()
    }
}

impl<T> Clone for CommandChannel<T> {
    fn clone(&self) -> Self {
        CommandChannel {
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
        }
    }
}

impl Default for CommandChannel<()> {
    fn default() -> Self {
        CommandChannel::new()
    }
}
