use crate::message::Message;
use crossbeam_channel::Sender;
use std::cell::RefCell;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct ActorAddress {
    pub(crate) uri: Uri,

    /// mailbox is a RefCell containing an optional sender. ActorAddresses may be created from
    /// just a path, but once a message is sent that path will need to resolve to a mailbox. Once
    /// the mailbox is resolved, it can be stored here for future use.
    pub(crate) mailbox: RefCell<Option<Sender<Box<dyn Message>>>>,
}

impl Display for ActorAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.uri)
    }
}

impl Clone for ActorAddress {
    fn clone(&self) -> Self {
        Self {
            uri: self.uri.clone(),
            mailbox: RefCell::new(self.mailbox.borrow().clone()),
        }
    }
}

impl ActorAddress {
    pub(crate) fn new_child(parent: &ActorAddress, name: &String, id: usize) -> Self {
        Self {
            uri: parent.uri.new_child(&format!("{}-{}", name, id)),
            mailbox: RefCell::new(None),
        }
    }

    pub(crate) fn new_root(name: &String) -> Self {
        Self {
            uri: Uri::new(UriScheme::Local, &[name]),
            mailbox: RefCell::new(None),
        }
    }

    pub(crate) fn set_mailbox(&self, mailbox: Sender<Box<dyn Message>>) {
        *self.mailbox.borrow_mut() = Some(mailbox);
    }

    pub(crate) fn is_resolved(&self) -> bool {
        self.mailbox.borrow().is_some()
    }

    pub(crate) fn send(&self, message: Box<dyn Message>) {
        let result = (self.mailbox.borrow().as_ref().unwrap()).send(message);
        // TODO: Handle a non-OK error (once actor shutdown is implemented) On error, should
        //       redirect to the dead letter queue. This function may simply return an error
        //       so that the caller can do the redirection.
        debug_assert!(result.is_ok(), "Error sending to actor address {}", self);
    }

    pub(crate) fn is_parent(&self, child: &ActorAddress) -> bool {
        self.uri.is_parent(&child.uri)
    }
}

/// `UriScheme` is the transport mechanism for messages sent between actor systems. Messages
/// that stay within the current actor system will all have a `Local` scheme.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum UriScheme {
    /// `Local` is the default scheme for messages that stay within the current actor system.
    Local,

    /// `Remote` is currently a placeholder value since remote is not currently implemented.
    Remote,
}

/// `Uri` is a URI-like type that identifies an actor system and an actor within that system.
/// The hierarchical nature, or tree-like, organization of actors is also present in URIs, with
/// children and parents readily identifiable by path. Take for example the following hierarchy
/// and their `Uri`s:
///
/// ```text
/// geoip_updater                    local://geoip_updater
/// ├── download_manager             local://geoip_updater/download_manager
/// │   ├── fetch-0                  local://geoip_updater/download_manager/fetch-0
/// │   ├── fetch-1                  local://geoip_updater/download_manager/fetch-1
/// ├── indexer_manager              local://geoip_updater/download_manager/index_manager
/// │   ├── indexer-0                local://geoip_updater/download_manager/index_manager/indexer-0
/// │   └── indexer-1                local://geoip_updater/download_manager/index_manager/indexer-1
/// └── publisher                    local://geoip_updater/download_manager/publisher
///     ├── change_log               local://geoip_updater/download_manager/publisher/change_log
///     ├── database                 local://geoip_updater/download_manager/publisher/database
///     └── event_emitter            local://geoip_updater/download_manager/publisher/event_emitter
/// ```
/// Because of this property, `Uri` has additional methods for creating child `Uri`s, parent
/// `Uri`s or identifying the relationship between two `Uri`s.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Uri {
    scheme: UriScheme,
    path_segments: Vec<String>,
}

impl Uri {
    fn new(scheme: UriScheme, path_segments: &[&str]) -> Self {
        Self {
            scheme,
            path_segments: path_segments.iter().map(|s| String::from(*s)).collect(),
        }
    }

    /// Construct a new `Uri` from `Self` with `sub_path` appended to the end of the path.
    fn new_child(&self, sub_path: &str) -> Self {
        let mut path_segments = self.path_segments.clone();
        path_segments.push(String::from(sub_path));
        Self {
            scheme: self.scheme.clone(),
            path_segments,
        }
    }

    /// Construct a new `Uri` from `Self` with the last path segment removed. If there is only
    /// one path segment (i.e. `Self` is the root), then effectively a copy of `Self` is returned.
    fn new_parent(&self) -> Self {
        let mut path_segments = self.path_segments.clone();
        if path_segments.len() > 1 {
            path_segments.pop();
        }
        Self {
            scheme: self.scheme.clone(),
            path_segments,
        }
    }

    /// Returns true if `Self` is the direct parent of `maybe_child`.
    fn is_child(&self, maybe_child: &Self) -> bool {
        if self.scheme != maybe_child.scheme {
            return false;
        }
        if self.path_segments.len() >= maybe_child.path_segments.len() {
            return false;
        }
        if self.path_segments.len() != maybe_child.path_segments.len() - 1 {
            return false;
        }
        for (i, segment) in self.path_segments.iter().enumerate() {
            if segment != &maybe_child.path_segments[i] {
                return false;
            }
        }
        true
    }

    // Returns true if `Self` is the direct child of `maybe_parent`.
    fn is_parent(&self, maybe_parent: &Self) -> bool {
        maybe_parent.is_child(self)
    }
}

impl Display for Uri {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.scheme {
            UriScheme::Local => write!(f, "local://")?,
            UriScheme::Remote => write!(f, "remote://")?,
        }
        write!(f, "{}", self.path_segments.join("/"))
    }
}
