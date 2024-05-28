// use walkdir::WalkDir;
use crate::cli::Args;

pub struct CoopWorkflow;

impl CoopWorkflow {
  pub fn new(args: Args) -> Self {
    println!("args: {:?}", args);
    Self
  }
}
