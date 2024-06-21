use std::fmt;
use std::path::Path;
use crate::{model::size_pretty, progress::MyProgressBar};

#[derive(Debug, Clone)]
pub enum FileStatus {
  NotStarted(MyProgressBar),
  OpenedSourceFile(MyProgressBar),
  GotDestinationFileLength(MyProgressBar),
  GettingDestinationFileLength(MyProgressBar),
  CreatedDestinationFile(MyProgressBar),
  InProgress(u64),
  CopyComplete(Complete),
  FileSizesMatch(MyProgressBar),
  Success(FileName, FileSize, MyProgressBar),
  Failed(FailedReason),
  Flushing(MyProgressBar)
}

#[derive(Debug, Clone)]
pub struct FileName(String);

impl FileName {
  pub fn new(file_name: &str) -> Self {
    Self(file_name.to_owned())
  }

  pub fn name(self) -> String {
    self.0
  }
}

impl <P: AsRef<Path>>From<P> for FileName {
  fn from(path: P) -> Self {
    FileName(path.as_ref().file_name().map(|p| p.to_string_lossy().to_string()).unwrap_or("<unknown>".to_owned()))
  }
}

#[derive(Debug, Clone)]
pub struct FileSize(u64);

impl FileSize {
  pub fn new(file_size: u64) -> Self {
    Self(file_size)
  }

  #[allow(dead_code)]
  pub fn size(self) -> u64 {
    self.0
  }
}

impl fmt::Display for FileSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", size_pretty(self.0))
    }
}


#[derive(Debug, Clone)]
pub struct CopyError(String);

impl CopyError {
  pub fn new(message: &str) -> Self {
    Self(message.to_owned())
  }

  pub fn message(self) -> String {
    self.0
  }

  pub fn error(self) -> String {
    self.0
  }
}

impl <T> From<T> for CopyError where
  T: std::error::Error
  {
    fn from(error: T) -> Self {
      CopyError(error.to_string())
  }
}

#[derive(Debug, Clone)]
pub struct InProgress {
  bytes_written: u64,
  progress_bar: MyProgressBar
}

impl InProgress {

  pub fn new(bytes_written: u64, progress_bar: &MyProgressBar) -> Self {
    Self {
      bytes_written,
      progress_bar: progress_bar.clone()
    }
  }

  pub fn progress_bar(self) -> MyProgressBar {
    self.progress_bar
  }

  pub fn bytes_written(&self) -> u64 {
    self.bytes_written
  }
}

#[derive(Debug, Clone)]
pub struct Complete {
  progress_bar: MyProgressBar
}

impl Complete {

  pub fn new(progress_bar: &MyProgressBar) -> Self {
    Self {
      progress_bar: progress_bar.clone()
    }
  }

  pub fn progress_bar(self) -> MyProgressBar {
    self.progress_bar
  }
}

#[derive(Debug, Clone)]
struct SouceSize(u64);

#[derive(Debug, Clone)]
struct DestinationSize(u64);

#[derive(Debug, Clone)]
pub struct SizeComparison {
  source: SouceSize,
  destination: DestinationSize
}

impl SizeComparison {

  pub fn new(source: u64, destination: u64) -> Self {
    Self {
      source: SouceSize(source),
      destination: DestinationSize(destination),
    }
  }

  pub fn source_size(&self) -> u64 {
    self.source.0
  }

  pub fn destination_size(&self) -> u64 {
    self.destination.0
  }
}

#[derive(Debug, Clone)]
pub enum FailedReason {
  ReadFailed(FileName, CopyError, MyProgressBar),
  WriteFailed(FileName, CopyError, MyProgressBar),
  FlushFailed(FileName, CopyError, MyProgressBar),
  CouldNotReadSourceFile(FileName, CopyError, MyProgressBar),
  CouldNotGetDestinationFileSize(FileName, CopyError, MyProgressBar),
  CouldNotCreateDestinationFile(FileName, CopyError, MyProgressBar),
  CouldNotCreateDestinationDir(FileName, CopyError, MyProgressBar),
  FileSizesAreDifferent(FileName, SizeComparison, MyProgressBar),
}
