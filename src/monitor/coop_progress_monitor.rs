use std::{sync::atomic::AtomicU64, thread, time::{Duration, Instant}};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tokio::sync::broadcast::Receiver;
use crate::model::{R, FileStatus};

pub struct CoopProgressMonitor {
  progress: ProgressBar,
  items: u64,
  completed: AtomicU64,
}

impl CoopProgressMonitor {

  pub fn new(multi: &MultiProgress, size: u64) -> Self {
    let progress_bar_style =
      ProgressStyle::with_template("[{msg}] overall progress: {prefix} [{wide_bar:.green}]").unwrap();

    let progress_bar =
      ProgressBar::new(size)
      .with_style(progress_bar_style)
      .with_finish(indicatif::ProgressFinish::Abandon);

    multi.add(progress_bar.clone());

    Self {
      progress: progress_bar,
      items: size,
      completed: AtomicU64::new(0)
    }
  }

  pub async fn monitor(mut self, mut rx: Receiver<FileStatus>, start_time: Instant) -> R<()> {
    self.progress.tick();
    let timer_handle = {
      let pb = self.progress.clone();
      let h = thread::spawn(move || {
        while !pb.is_finished() {
          let current_time = Instant::now();
          let duration = current_time.duration_since(start_time);
          let seconds = duration.as_secs();
          let minutes = seconds / 60;
          let hours = minutes / 60;
          pb.set_message(format!("{:02}:{:02}:{:02}", hours, minutes, seconds));
          thread::sleep(Duration::from_secs(1));
        }
      });
      h
    };

    while let Ok(value) = rx.recv().await {
      match value {
        FileStatus::CopyComplete(..) => {
          let completed = self.completed.get_mut();
          *completed += 1;
          self.progress.inc(1);
          self.progress.set_prefix(format!("{}/{}", completed, self.items));

          // If all items are completed, then finish
          if *completed >= self.items {
            self.progress.finish()
          }
        },

        FileStatus::Failed(..) => {
          let completed = self.completed.get_mut();
          *completed += 1;
          self.progress.inc(1);

          // If all items are completed, then finish
          if *completed >= self.items {
            self.progress.finish()
          }
        },

        _ => ()
     }
    }

    let _ = timer_handle.join();

    Ok(())
  }
}

