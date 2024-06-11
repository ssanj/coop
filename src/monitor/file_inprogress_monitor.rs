use tokio::sync::mpsc::Receiver;
use crate::model::{InProgress, R};

/// Monitors InProgress messages when a file copy is underway.
pub struct FileInProgressMonitor;

impl FileInProgressMonitor {

  /// This is a high cardinality event receiver.
  pub async fn monitor(mut rx: Receiver<InProgress>) -> R<()> {
    while let Some(progress) = rx.recv().await {
      let bytes_written = progress.bytes_written();
      let pb = progress.progress_bar();
      pb.update_progress(bytes_written);
      pb.set_status("copying...");
    }

    Ok(())
  }
}

