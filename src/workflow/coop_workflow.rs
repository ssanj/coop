// use walkdir::WalkDir;
use crate::cli::Args;

pub struct CoopWorkflow {
  args: Args
}

impl CoopWorkflow {
  pub fn new(args: Args) -> Self {
    Self {
      args
    }
  }

  pub fn run(self) {
    println!("args: {:?}", self.args);
  }
}
