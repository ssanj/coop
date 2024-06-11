use tokio::sync::broadcast::{self};
use tokio::sync::mpsc::{self};

use crate::model::{Complete, CopyError, FailedReason, FileName, FileStatus, FileType, InProgress, SizeComparison};
use crate::progress::MyProgressBar;

#[derive(Debug, Clone)]
pub struct MonitorMux {
  tx: broadcast::Sender<FileStatus>,
  txp: mpsc::Sender<InProgress>,
}

impl MonitorMux {

  pub fn new(broadcast_sender: broadcast::Sender<FileStatus>, mpsc_sender: mpsc::Sender<InProgress>) -> Self {
    Self {
      tx: broadcast_sender,
      txp: mpsc_sender
    }
  }

  pub fn send_opened_source_file(&self, progress_bar: &MyProgressBar) {
    let _ = &self.tx.send(FileStatus::OpenedSourceFile(progress_bar.clone()));
  }

  pub fn send_could_not_read_source_file<F: Into<FileName>, E: Into<CopyError>>(&self, file: F, error: E, progress_bar: &MyProgressBar) {
    let _ =
      &self.tx.send(
        FileStatus::Failed(
          FailedReason::CouldNotReadSourceFile(
            file.into(),
            error.into(),
            progress_bar.clone()
          )
        )
      );
  }

  pub fn send_getting_file_length(&self, file_type: &FileType, progress_bar: &MyProgressBar) {
    let _ = self.tx.send(FileStatus::GettingFileLength(file_type.clone(), progress_bar.clone()));
  }

  pub fn send_got_file_length(&self, file_type: &FileType, progress_bar: &MyProgressBar) {
    let _ = self.tx.send(FileStatus::GotFileLength(file_type.clone(), progress_bar.clone()));
  }

  pub fn send_could_not_get_file_size<E: Into<CopyError>>(&self, file_name: &str, file_type: &FileType, error: E, progress_bar: &MyProgressBar) {
    let _ = self.tx.send(
      FileStatus::Failed(
        FailedReason::CouldNotGetFileSize(
          FileName::new(file_name),
          error.into(),
          file_type.clone(),
          progress_bar.clone()
        )
      )
    );
  }

  pub fn send_not_started(&self, progress_bar: &MyProgressBar) {
    let _ = self.tx.send(FileStatus::NotStarted(progress_bar.clone()));
  }

  pub fn send_could_not_create_destination_directory<P: AsRef<std::path::Path>, E: Into<CopyError>>(&self, destination_file: P, error: E, progress_bar: &MyProgressBar) {
    let _ =
      self.tx.send(
        FileStatus::Failed(
          FailedReason::CouldNotCreateDestinationDir(
            destination_file.into(),
            error.into(),
            progress_bar.clone()
          )
        )
      );
  }

  pub fn send_created_destination_file(&self, progress_bar: &MyProgressBar) {
    let _ = self.tx.send(FileStatus::CreatedDestinationFile(progress_bar.clone()));
  }

  pub fn send_could_not_create_destination_file<P: AsRef<std::path::Path>, E: Into<CopyError>>(&self, destination_file: P, error: E, progress_bar: &MyProgressBar) {
    let _ =
      self.tx.send(
        FileStatus::Failed(
          FailedReason::CouldNotCreateDestinationFile(
            destination_file.into(),
            error.into(),
            progress_bar.clone()
          )
        )
      );
  }

  pub fn send_read_failed<E : Into<CopyError>>(&self, file: &str, error: E, progress_bar: &MyProgressBar) {
    let _ =
      self.tx.send(
        FileStatus::Failed(
          FailedReason::ReadFailed(
            FileName::new(file),
            error.into(),
            progress_bar.clone())
        )
      );
  }

  pub fn send_flushing_destination_file(&self, progress_bar: &MyProgressBar) {
    let _ = self.tx.send(FileStatus::Flushing(progress_bar.clone()));
  }

  pub fn send_flushing_to_destination_file_failed<E : Into<CopyError>>(&self, file: &str, error: E, progress_bar: &MyProgressBar) {
    let _ =
      self.tx.send(
        FileStatus::Failed(
          FailedReason::FlushFailed(
            FileName::new(file),
            error.into(),
            progress_bar.clone())
        )
      );
  }

  pub fn send_copy_complete(&self, progress_bar: &MyProgressBar) {
    let _ = self.tx.send(FileStatus::CopyComplete(Complete::new(progress_bar)));
  }

  pub fn send_file_sizes_match(&self, progress_bar: &MyProgressBar) {
    let _ = self.tx.send(FileStatus::FileSizesMatch(progress_bar.clone()));
  }

  pub fn send_files_sizes_are_different(&self, file: &str, size_comparison: SizeComparison, progress_bar: &MyProgressBar) {
    let _ =
      self.tx.send(
        FileStatus::Failed(
          FailedReason::FileSizesAreDifferent(
            FileName::new(file),
            size_comparison,
            progress_bar.clone()
          )
        )
      );
  }

  pub fn send_success(&self, file_name: &str, progress_bar: &MyProgressBar) {
    let _ = self.tx.send(FileStatus::Success(FileName::new(file_name), progress_bar.clone()));
  }

  pub fn send_write_to_destination_failed<E : Into<CopyError>>(&self, file: &str, error: E, progress_bar: &MyProgressBar) {
    let _ =
      self.tx.send(
        FileStatus::Failed(
          FailedReason::WriteFailed(
            FileName::new(file),
            error.into(),
            progress_bar.clone()
          )
        )
      );
  }

  pub async fn send_copy_in_progress(&self, bytes_written: u64, progress_bar: &MyProgressBar) {
    let _ = self.txp.send(InProgress::new(bytes_written, progress_bar)).await;
  }
}
