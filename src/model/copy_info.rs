use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct CopyInfo {
  source_file: PathBuf,
  destination_file: PathBuf,
}

impl CopyInfo {
  pub fn new<S: AsRef<Path>, D: AsRef<Path>>(source: S, destination: D) -> Self {
    Self {
      source_file: source.as_ref().to_path_buf(),
      destination_file: destination.as_ref().to_path_buf(),
    }
  }

  pub fn source_file(&self) -> &Path {
    self.source_file.as_path()
  }

  pub fn destination_file(&self) -> &Path {
    &self.destination_file.as_path()
  }
}
