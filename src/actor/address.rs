use crate::actor::{Letter, Mailbox};
use crate::message::Message;
use log::trace;
use std::cell::RefCell;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct ActorAddress {
    pub(crate) uri: Uri,

    /// `mailbox` is a `Sender` channel used for transmitting messages to the actor and is
    /// is implemented as a `RefCell` for interior mutability. This allows for addresses creation
    /// to be decoupled from resolution of the mailbox.
    pub(crate) mailbox: RefCell<Option<Mailbox>>,
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

    pub(crate) fn set_mailbox(&self, mailbox: Mailbox) {
        *self.mailbox.borrow_mut() = Some(mailbox);
    }

    pub(crate) fn is_resolved(&self) -> bool {
        self.mailbox.borrow().is_some()
    }

    pub(crate) fn send(&self, from: Option<Self>, message: Box<dyn Message>) {
        trace!(
            "[{}] Sending message to {}",
            (&from.as_ref())
                .map(|from| format!("{}", from))
                .unwrap_or_else(|| "".to_string()),
            self
        );

        let letter = Letter::new(from, self, message);
        let result = (self.mailbox.borrow().as_ref().unwrap()).send(letter);
        // TODO: Handle a non-OK error (once actor shutdown is implemented) On error, should
        //       redirect to the dead letter queue. This function may simply return an error
        //       so that the caller can do the redirection.
        debug_assert!(result.is_ok(), "Error sending to actor address {}", self);
    }

    pub(crate) fn is_parent(&self, maybe_parent: &ActorAddress) -> bool {
        self.uri.is_parent(&maybe_parent.uri)
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
        if path_segments.is_empty() {
            panic!("Uri must have at least one path segment");
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_construction() {
        // Test that an empty path is not allowed
        Uri::new(UriScheme::Local, &[]);
    }

    #[test]
    fn test_child_construction() {
        // Create a child from a root path
        let root = Uri::new(UriScheme::Local, &["root"]);
        let child = root.new_child("child");

        // Test the relationships between the two
        assert_eq!(root.is_child(&child), true);
        assert_eq!(child.is_parent(&root), true);

        assert_eq!(root.is_parent(&child), false);
        assert_eq!(child.is_child(&root), false);

        // Create a grandchild from a child
        let grandchild = child.new_child("grandchild");

        // Test the relationships between the grandchild and the root. Since we're not looking
        // at direct relationships, we expect the results to be false.
        assert_eq!(root.is_child(&grandchild), false);
        assert_eq!(grandchild.is_parent(&root), false);
    }

    #[test]
    fn test_parent_construction() {
        // Start from the bottom and create the parent and grandparent
        let child = Uri::new(UriScheme::Local, &["grandparent", "parent", "child"]);
        let parent = child.new_parent();
        let grandparent = parent.new_parent();

        // Test the relationship between the child and parent
        assert_eq!(child.is_parent(&parent), true);
        assert_eq!(parent.is_child(&child), true);
        assert_eq!(child.is_child(&parent), false);
        assert_eq!(parent.is_parent(&child), false);

        // Test the relationship between the child and grandparent Since we're not looking
        // at direct relationships, we expect the results to be false.
        assert_eq!(child.is_parent(&grandparent), false);
        assert_eq!(grandparent.is_child(&child), false);
    }

    #[test]
    fn test_self_reference() {
        let path = Uri::new(UriScheme::Local, &["root", "some", "path"]);
        assert_eq!(path.is_child(&path), false);
        assert_eq!(path.is_parent(&path), false);
    }

    #[test]
    fn test_display() {
        let test_cases = vec![
            (vec!["geoip_updater"], "local://geoip_updater"),
            (
                vec!["geoip_updater", "download_manager"],
                "local://geoip_updater/download_manager",
            ),
            (
                vec!["geoip_updater", "download_manager", "fetch-0"],
                "local://geoip_updater/download_manager/fetch-0",
            ),
            (
                vec!["geoip_updater", "download_manager", "fetch-1"],
                "local://geoip_updater/download_manager/fetch-1",
            ),
            (
                vec!["geoip_updater", "indexer_manager"],
                "local://geoip_updater/indexer_manager",
            ),
            (
                vec!["geoip_updater", "indexer_manager", "indexer-0"],
                "local://geoip_updater/indexer_manager/indexer-0",
            ),
            (
                vec!["geoip_updater", "indexer_manager", "indexer-1"],
                "local://geoip_updater/indexer_manager/indexer-1",
            ),
            (
                vec!["geoip_updater", "publisher"],
                "local://geoip_updater/publisher",
            ),
            (
                vec!["geoip_updater", "publisher", "change_log"],
                "local://geoip_updater/publisher/change_log",
            ),
            (
                vec!["geoip_updater", "publisher", "database"],
                "local://geoip_updater/publisher/database",
            ),
            (
                vec!["geoip_updater", "publisher", "event_emitter"],
                "local://geoip_updater/publisher/event_emitter",
            ),
        ];
        for (path_segments, expected) in test_cases {
            let uri = Uri::new(UriScheme::Local, &path_segments);
            assert_eq!(uri.to_string(), expected);
        }
    }
}
