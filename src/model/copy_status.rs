use std::path::Path;

use clap::Error;
use serde_json::error;

use crate::progress::MyProgressBar;

#[derive(Debug, Clone)]
pub enum FileStatus {
  NotStarted(MyProgressBar),
  OpenedSourceFile(MyProgressBar),
  GotFileLength(FileType, MyProgressBar),
  GettingFileLength(FileType, MyProgressBar),
  CreatedDestinationFile(MyProgressBar),
  CopyInProgress(InProgress),
  CopyComplete(Complete),
  FileSizesMatch(MyProgressBar),
  Success(FileName, MyProgressBar),
  Failed(FailedReason),
  Flushing(MyProgressBar)
}

// TODO: Add implementation functions instead of sharing fields
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
    FileName(path.as_ref().to_string_lossy().to_string())
  }
}

#[derive(Debug, Clone)]
pub struct CopyError(String);

impl CopyError {
  pub fn new(message: &str) -> Self {
    Self(message.to_owned())
  }

  // TODO: Remove, use error instead
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
pub enum FileType {
  Source,
  Destination,
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
  CouldNotGetFileSize(FileName, CopyError, FileType, MyProgressBar),
  CouldNotCreateDestinationFile(FileName, CopyError, MyProgressBar),
  CouldNotCreateDestinationDir(FileName, CopyError, MyProgressBar),
  FileSizesAreDifferent(FileName, SizeComparison, MyProgressBar),
}
