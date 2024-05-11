/// Error type for all errors originating from library actions
#[derive(Debug)]
pub enum BusanError {
    /// Encountered when an actor is unassignable for any reason (e.g. no
    /// executors available, IO error when scheduling remotely). Contains
    /// a user focused explanation of the specific error cause.
    UnassignableActor(String),
}
