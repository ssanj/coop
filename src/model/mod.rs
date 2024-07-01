use std::fmt;

mod copy_status;
mod sizes;

pub type R<A> = Result<A, ()>;

pub use copy_status::*;
pub use sizes::*;

#[derive(Debug)]
pub enum CoopError {
  CouldNotOpenLogFile(String)
}

impl fmt::Display for CoopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let error = match self {
        CoopError::CouldNotOpenLogFile(e) => format!("Could not open coop.log due to: {e}"),
      };

      write!(f, "{}", &error)
    }
}
