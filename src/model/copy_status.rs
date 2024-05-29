use crate::progress::MyProgressBar;

pub enum FileStatus {
  NotStarted(MyProgressBar),
  OpenedSourceFile(MyProgressBar),
  GotFileLength(FileType, MyProgressBar),
  GettingFileLength(FileType, MyProgressBar),
  CreatedDestinationFile(MyProgressBar),
  CopyInProgress(InProgress),
  CopyComplete(Complete),
  FileSizesMatch(MyProgressBar),
  Failed(FailedReason),
  Flushing(MyProgressBar)
}

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

#[derive(Clone)]
pub enum FileType {
  Source,
  Destination,
}

pub enum FailedReason {
  ReadFailed(String, MyProgressBar),
  WriteFailed(String, MyProgressBar),
  FlushFailed(String, MyProgressBar),
  CouldNotReadSourceFile(String, MyProgressBar),
  CouldNotGetFileSize(String, FileType, MyProgressBar),
  CouldNotCreateDestinationFile(String, MyProgressBar),
  CouldNotCreateDestinationDir(String, MyProgressBar),
  FileSizesAreDifferent(u64, u64, MyProgressBar),
}
