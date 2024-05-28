mod copy_status;
mod copy_info;

pub type R<A> = Result<A, ()>;

pub use copy_status::*;
pub use copy_info::CopyInfo;
