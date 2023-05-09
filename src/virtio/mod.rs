pub mod block;
pub mod header;
pub mod queue;

#[derive(Debug)]
pub enum Error {
    /// Device status not OK
    HeaderInitError,
    /// The queue is already in use.
    AlreadyUsed,
    /// The queue is not available
    NotAvailable,
    /// Invalid arguments
    InvalidArguments,
    /// Cannot get memory
    NoMemory,
}
