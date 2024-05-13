use std::fmt::Display;

/// Error type for all errors originating from library actions
#[derive(Debug)]
pub enum BusanError {
    /// Encountered when an actor is unassignable for any reason (e.g. no
    /// executors available, IO error when scheduling remotely). Contains
    /// a user focused explanation of the specific error cause.
    UnassignableActor(String),
}

impl Display for BusanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BusanError::UnassignableActor(s) => write!(f, "Actor is unassignable: {}", s),
        }
    }
}
