use std::ops::Deref;

use tokio::sync::mpsc::{self};

use crate::model::{Complete, CopyError, FailedReason, FileName, FileSize, FileStatus, InProgress, SizeComparison};
use crate::progress::MyProgressBar;

#[derive(Debug)]
pub struct LifecycleEventSender(mpsc::Sender<FileStatus>);

impl LifecycleEventSender {
  pub fn new(inner: mpsc::Sender<FileStatus>) -> Self {
    Self(inner)
  }
}

impl Deref for LifecycleEventSender {
  type Target = mpsc::Sender<FileStatus>;

  fn deref(&self) -> &Self::Target {
      &self.0
  }
}

#[derive(Debug)]
pub struct OverallProgressSender(mpsc::Sender<FileStatus>);

impl OverallProgressSender {
  pub fn new(inner: mpsc::Sender<FileStatus>) -> Self {
    Self(inner)
  }
}


impl Deref for OverallProgressSender {
  type Target = mpsc::Sender<FileStatus>;

  fn deref(&self) -> &Self::Target {
      &self.0
  }
}

#[derive(Debug)]
pub struct InProgressSender(mpsc::Sender<InProgress>);

impl InProgressSender {
  pub fn new(inner: mpsc::Sender<InProgress>) -> Self {
    Self(inner)
  }

}

impl Deref for InProgressSender {
  type Target = mpsc::Sender<InProgress>;

  fn deref(&self) -> &Self::Target {
      &self.0
  }
}


#[derive(Debug)]
pub struct MonitorMux {
  lifecycle_event_sender: LifecycleEventSender,
  overall_progress_sender: OverallProgressSender,
  inprogress_sender: InProgressSender,
}

impl MonitorMux {

  pub fn new(
    inprogress_sender: InProgressSender,
    lifecycle_event_sender: LifecycleEventSender,
    overall_progress_sender: OverallProgressSender
    ) -> Self {
    Self {
      lifecycle_event_sender,
      overall_progress_sender,
      inprogress_sender
    }
  }

  pub async fn send_opened_source_file(&self, progress_bar: &MyProgressBar) {
    let _ = &self.lifecycle_event_sender.send(FileStatus::OpenedSourceFile(progress_bar.clone())).await;
    let _ = &self.overall_progress_sender.send(FileStatus::OpenedSourceFile(progress_bar.clone())).await;
  }

  pub async fn send_could_not_read_source_file<F: Into<FileName> + Clone, E: Into<CopyError> + Clone>(&self, file: F, error: E, progress_bar: &MyProgressBar) {
    let _ =
      &self.lifecycle_event_sender.send(
        FileStatus::Failed(
          FailedReason::CouldNotReadSourceFile(
            file.clone().into(),
            error.clone().into(),
            progress_bar.clone()
          )
        )
      ).await;

    let _ =
      &self.overall_progress_sender.send(
        FileStatus::Failed(
          FailedReason::CouldNotReadSourceFile(
            file.into(),
            error.into(),
            progress_bar.clone()
          )
        )
      ).await;
  }

  pub async fn send_getting_file_length(&self, progress_bar: &MyProgressBar) {
    let _ = self.lifecycle_event_sender.send(FileStatus::GettingDestinationFileLength(progress_bar.clone())).await;
    let _ = self.overall_progress_sender.send(FileStatus::GettingDestinationFileLength(progress_bar.clone())).await;
  }

  pub async fn send_got_file_length(&self, progress_bar: &MyProgressBar) {
    let _ = self.lifecycle_event_sender.send(FileStatus::GotDestinationFileLength(progress_bar.clone())).await;
    let _ = self.overall_progress_sender.send(FileStatus::GotDestinationFileLength(progress_bar.clone())).await;
  }

  pub async fn send_could_not_get_destination_file_size<E: Into<CopyError> + Clone>(&self, file_name: &str, error: E, progress_bar: &MyProgressBar) {
    let _ = self.lifecycle_event_sender.send(
      FileStatus::Failed(
        FailedReason::CouldNotGetDestinationFileSize(
          FileName::new(file_name),
          error.clone().into(),
          progress_bar.clone()
        )
      )
    ).await;

    let _ = self.overall_progress_sender.send(
      FileStatus::Failed(
        FailedReason::CouldNotGetDestinationFileSize(
          FileName::new(file_name),
          error.into(),
          progress_bar.clone()
        )
      )
    ).await;
  }

  pub async fn send_not_started(&self, progress_bar: &MyProgressBar) {
    let _ = self.lifecycle_event_sender.send(FileStatus::NotStarted(progress_bar.clone())).await;
    let _ = self.overall_progress_sender.send(FileStatus::NotStarted(progress_bar.clone())).await;
  }

  pub async fn send_could_not_create_destination_directory<P: AsRef<std::path::Path> + Clone, E: Into<CopyError> + Clone>(&self, destination_file: P, error: E, progress_bar: &MyProgressBar) {
    let _ =
      self.lifecycle_event_sender.send(
        FileStatus::Failed(
          FailedReason::CouldNotCreateDestinationDir(
            destination_file.clone().into(),
            error.clone().into(),
            progress_bar.clone()
          )
        )
      ).await;

    let _ =
      self.overall_progress_sender.send(
        FileStatus::Failed(
          FailedReason::CouldNotCreateDestinationDir(
            destination_file.into(),
            error.into(),
            progress_bar.clone()
          )
        )
      ).await;
  }

  pub async fn send_created_destination_file(&self, progress_bar: &MyProgressBar) {
    let _ = self.lifecycle_event_sender.send(FileStatus::CreatedDestinationFile(progress_bar.clone())).await;
    let _ = self.overall_progress_sender.send(FileStatus::CreatedDestinationFile(progress_bar.clone())).await;
  }

  pub async fn send_could_not_create_destination_file<P: AsRef<std::path::Path> + Clone, E: Into<CopyError> + Clone>(&self, destination_file: P, error: E, progress_bar: &MyProgressBar) {
    let _ =
      self.lifecycle_event_sender.send(
        FileStatus::Failed(
          FailedReason::CouldNotCreateDestinationFile(
            destination_file.clone().into(),
            error.clone().into(),
            progress_bar.clone()
          )
        )
      ).await;

    let _ =
      self.overall_progress_sender.send(
        FileStatus::Failed(
          FailedReason::CouldNotCreateDestinationFile(
            destination_file.into(),
            error.into(),
            progress_bar.clone()
          )
        )
      ).await;
  }

  pub async fn send_read_failed<E : Into<CopyError> + Clone>(&self, file: &str, error: E, progress_bar: &MyProgressBar) {
    let _ =
      self.lifecycle_event_sender.send(
        FileStatus::Failed(
          FailedReason::ReadFailed(
            FileName::new(file),
            error.clone().into(),
            progress_bar.clone())
        )
      ).await;

    let _ =
      self.overall_progress_sender.send(
        FileStatus::Failed(
          FailedReason::ReadFailed(
            FileName::new(file),
            error.into(),
            progress_bar.clone())
        )
      ).await;
  }

  pub async fn send_flushing_destination_file(&self, progress_bar: &MyProgressBar) {
    let _ = self.lifecycle_event_sender.send(FileStatus::Flushing(progress_bar.clone())).await;
    let _ = self.overall_progress_sender.send(FileStatus::Flushing(progress_bar.clone())).await;
  }

  pub async fn send_flushing_to_destination_file_failed<E : Into<CopyError> + Clone>(&self, file: &str, error: E, progress_bar: &MyProgressBar) {
    let _ =
      self.lifecycle_event_sender.send(
        FileStatus::Failed(
          FailedReason::FlushFailed(
            FileName::new(file),
            error.clone().into(),
            progress_bar.clone())
        )
      ).await;

    let _ =
      self.overall_progress_sender.send(
        FileStatus::Failed(
          FailedReason::FlushFailed(
            FileName::new(file),
            error.into(),
            progress_bar.clone())
        )
      ).await;
  }

  pub async fn send_copy_complete(&self, progress_bar: &MyProgressBar) {
    let _ = self.lifecycle_event_sender.send(FileStatus::CopyComplete(Complete::new(progress_bar))).await;
    let _ = self.overall_progress_sender.send(FileStatus::CopyComplete(Complete::new(progress_bar))).await;
  }

  pub async fn send_file_sizes_match(&self, progress_bar: &MyProgressBar) {
    let _ = self.lifecycle_event_sender.send(FileStatus::FileSizesMatch(progress_bar.clone())).await;
    let _ = self.overall_progress_sender.send(FileStatus::FileSizesMatch(progress_bar.clone())).await;
  }

  pub async fn send_files_sizes_are_different(&self, file: &str, size_comparison: SizeComparison, progress_bar: &MyProgressBar) {
    let _ =
      self.lifecycle_event_sender.send(
        FileStatus::Failed(
          FailedReason::FileSizesAreDifferent(
            FileName::new(file),
            size_comparison.clone(),
            progress_bar.clone()
          )
        )
      ).await;

    let _ =
      self.overall_progress_sender.send(
        FileStatus::Failed(
          FailedReason::FileSizesAreDifferent(
            FileName::new(file),
            size_comparison,
            progress_bar.clone()
          )
        )
      ).await;
  }

  pub async fn send_success(&self, file_name: &str, file_size: u64, progress_bar: &MyProgressBar) {
    let _ = self.lifecycle_event_sender.send(FileStatus::Success(FileName::new(file_name), FileSize::new(file_size), progress_bar.clone())).await;
    let _ = self.overall_progress_sender.send(FileStatus::Success(FileName::new(file_name), FileSize::new(file_size), progress_bar.clone())).await;
  }

  pub async fn send_write_to_destination_failed<E : Into<CopyError> + Clone>(&self, file: &str, error: E, progress_bar: &MyProgressBar) {
    let _ =
      self.lifecycle_event_sender.send(
        FileStatus::Failed(
          FailedReason::WriteFailed(
            FileName::new(file),
            error.clone().into(),
            progress_bar.clone()
          )
        )
      ).await;

    let _ =
      self.overall_progress_sender.send(
        FileStatus::Failed(
          FailedReason::WriteFailed(
            FileName::new(file),
            error.into(),
            progress_bar.clone()
          )
        )
      ).await;
  }

  pub async fn send_copy_in_progress(&self, bytes_written: u64, progress_bar: &MyProgressBar) {
    let _ = self.inprogress_sender.send(InProgress::new(bytes_written, progress_bar)).await;
    // Only send this to the overall progress monitor
    let _ = self.overall_progress_sender.send(FileStatus::InProgress(bytes_written)).await;
  }
}
