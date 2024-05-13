//! The CommandChannel is a wrapper around channels that provides a simple interface for an
//! unbounded channel. This is mostly used internally, but is exposed for use in any extension
//! points within busan and the execution model is guaranteed to work with the underlying channel
//! implementation.

use crossbeam_channel::{Receiver, Sender};

/// A simple wrapper for sending and receiving objects/commands over a channel. This object
/// represents both a sender and receiver for a channel.
pub struct CommandChannel<T> {
    pub(self) sender: Sender<T>,
    pub(self) receiver: Receiver<T>,
}

impl<T> CommandChannel<T> {
    pub fn new() -> CommandChannel<T> {
        let (sender, receiver) = crossbeam_channel::unbounded();
        CommandChannel { sender, receiver }
    }

    // TODO: Wrap the error type to avoid exposing the underlying channel implementation
    pub fn send(&self, command: T) -> Result<(), crossbeam_channel::SendError<T>> {
        self.sender.send(command)
    }

    // TODO: Wrap the error type to avoid exposing the underlying channel implementation
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
