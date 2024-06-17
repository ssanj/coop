use tokio::sync::mpsc::Receiver;
use crate::model::{FileStatus, FileType, FailedReason, R};

/// Monitors lifecyle events of all file copies in progress.
pub struct LifecycleEventMonitor;

impl LifecycleEventMonitor {

   /// This is a low cardinality event receiver.
   pub async fn monitor(mut rx: Receiver<FileStatus>) -> R<()>  {
      while let Some(value) = rx.recv().await {
        match value {
          FileStatus::NotStarted(pb) => pb.set_status("waiting..."),
          FileStatus::OpenedSourceFile(pb) => pb.set_status("opened source file"),
          FileStatus::GettingFileLength(FileType::Source, pb) => pb.set_status("getting source file length"),
          FileStatus::GotFileLength(FileType::Source, pb) => pb.set_status("calculated source file length"),
          FileStatus::GettingFileLength(FileType::Destination, pb) => pb.set_status("getting destination file length"),
          FileStatus::GotFileLength(FileType::Destination, pb) => pb.set_status("calculated destination file length"),
          FileStatus::CreatedDestinationFile(pb) => pb.set_status("created destination file"),
          FileStatus::Flushing(pb) => pb.set_status("flushing destination..."),

          FileStatus::CopyComplete(complete) => {
            let pb = complete.progress_bar();
            pb.set_status("finished copying")
          },

          FileStatus::FileSizesMatch(pb) => {
            pb.complete("verification complete ✅");
          },

          FileStatus::Success(..) => (),

          FileStatus::Failed(FailedReason::ReadFailed(_, reason, pb)) => {
            pb.set_error(&format!("❌ Read failed: {}", reason.message()))
          },

          FileStatus::Failed(FailedReason::WriteFailed(_, reason, pb)) => {
            pb.set_error(&format!("❌ Write failed: {}", reason.message()))
          },

          FileStatus::Failed(FailedReason::CouldNotGetFileSize(_, reason, FileType::Source, pb)) => {
            pb.set_status("calculating source file length...");
            pb.set_error(&format!("❌ Could not get source file size: {}", reason.error()))
          },

          FileStatus::Failed(FailedReason::CouldNotGetFileSize(_, reason, FileType::Destination, pb)) => {
            pb.set_status("calculating destination file length...");
            pb.set_error(&format!("❌ Could not get destination file size: {}", reason.error()))
          },

          FileStatus::Failed(FailedReason::CouldNotCreateDestinationFile(_, reason, pb)) => {
            pb.set_status("creating destination file...");
            pb.set_error(&format!("❌ Could not create destination file: {}", reason.error()))
          },

          FileStatus::Failed(FailedReason::CouldNotCreateDestinationDir(_, reason, pb)) => {
            pb.set_status("creating destination dir...");
            pb.set_error(&format!("❌ Could not create destination dir: {}", reason.error()))
          },

          FileStatus::Failed(FailedReason::CouldNotReadSourceFile(_, reason, pb)) => {
            pb.set_status("opening source file...");
            pb.set_error(&format!("❌ Could not read source file: {}", reason.error()))
          },

          FileStatus::Failed(FailedReason::FileSizesAreDifferent(_, comparison, pb)) => {
            pb.set_status("comparing source and destination file sizes...");
            pb.set_error(&format!("❌ File sizes are different. src:{}, dst:{}", comparison.source_size(), comparison.destination_size()))
          },

          FileStatus::Failed(FailedReason::FlushFailed(_, reason, pb)) => {
            pb.set_status("flushing destination to disk...");
            pb.set_error(&format!("❌ Flushing destination file failed: {}", reason.error()))
          },
        }
      }

    Ok(())
  }
}

