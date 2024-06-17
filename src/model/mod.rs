mod copy_status;
mod sizes;

pub type R<A> = Result<A, ()>;

pub use copy_status::*;
pub use sizes::*;
