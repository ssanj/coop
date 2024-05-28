use tokio::sync::mpsc::Receiver;
use crate::model::{R, FileStatus, FileType, FailedReason};

pub struct FileCopyProgressMonitor;

impl FileCopyProgressMonitor {

   async fn monitor(mut rx: Receiver<FileStatus>) -> R<()> {
      while let Some(value) = rx.recv().await {
        match value {
          FileStatus::NotStarted(pb) => pb.set_status("waiting..."),
          FileStatus::OpenedSourceFile(pb) => pb.set_status("opened source file"),
          FileStatus::GotFileLength(FileType::Source, pb) => pb.set_status("calculated source file length"),
          FileStatus::GotFileLength(FileType::Destination, pb) => pb.set_status("calculated destination file length"),
          FileStatus::CreatedDestinationFile(pb) => pb.set_status("created destination file"),

          FileStatus::CopyInProgress(progress) => {
              let bytes_written = progress.bytes_written();
              let pb = progress.progress_bar();
              pb.update_progress(bytes_written);
              pb.set_status("copying...");
          },

          FileStatus::CopyComplete(complete) => {
            let pb = complete.progress_bar();
            pb.complete("finished copying")
          },

          FileStatus::FileSizesMatch(pb) => {
            pb.complete("verification complete")
          },

          FileStatus::Failed(FailedReason::ReadFailed(reason, pb)) => {
            pb.set_error(&format!("Read failed: {}", reason))
          },

          FileStatus::Failed(FailedReason::WriteFailed(reason, pb)) => {
            pb.set_error(&format!("Write failed: {}", reason))
          },

          FileStatus::Failed(FailedReason::CouldNotGetFileSize(reason, FileType::Source, pb)) => {
            pb.set_status("calculating source file length...");
            pb.set_error(&format!("Could not get source file size: {}", reason))
          },

          FileStatus::Failed(FailedReason::CouldNotGetFileSize(reason, FileType::Destination, pb)) => {
            pb.set_status("calculating destination file length...");
            pb.set_error(&format!("Could not get destination file size: {}", reason))
          },

          FileStatus::Failed(FailedReason::CouldNotCreateDestinationFile(reason, pb)) => {
            pb.set_status("creating destination file...");
            pb.set_error(&format!("Could not create destination file: {}", reason))
          },

          FileStatus::Failed(FailedReason::CouldNotCreateDestinationDir(reason, pb)) => {
            pb.set_status("creating destination dir...");
            pb.set_error(&format!("Could not create destination dir: {}", reason))
          },

          FileStatus::Failed(FailedReason::CouldNotReadSourceFile(reason, pb)) => {
            pb.set_status("opening source file...");
            pb.set_error(&format!("Could not read source file: {}", reason))
          },

          FileStatus::Failed(FailedReason::FileSizesAreDifferent(source, dest, pb)) => {
            pb.set_status("comparing source and destination file sizes...");
            pb.set_error(&format!("File sizes are different. src:{source}, dst:{dest}"))
          },

          FileStatus::Failed(FailedReason::FlushFailed(reason, pb)) => {
            pb.set_status("flushing destination to disk...");
            pb.set_error(&format!("Flushing destination file failed: {}", reason))
          },
        }
      }

      Ok(())
  }
}

