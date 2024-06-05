use std::{sync::atomic::AtomicU64, thread, time::{Duration, Instant}};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tokio::sync::broadcast::Receiver;
use std::sync::atomic;

use crate::model::{R, FileStatus};

pub struct CoopProgressMonitor {
  progress: ProgressBar,
  items: u64,
  completed: AtomicU64,
  completed_index: AtomicU64,
  completed_items: Vec<ProgressBar>,
}

impl CoopProgressMonitor {

  pub fn new(multi: &MultiProgress, size: u64) -> Self {
    let completed_items =
      (0..size)
        .into_iter()
        .map(|_| {
          let pb = Self::create_completed_progress_bar();
          multi.add(pb.clone())
        })
        .collect();

    let overall_bar =
      ProgressStyle::with_template("[{msg}] overall progress: {prefix} [{wide_bar:.green}]").unwrap();

    let overall_bar =
      ProgressBar::new(size)
      .with_style(overall_bar)
      .with_finish(indicatif::ProgressFinish::Abandon);

    // Add this at the end
    multi.add(overall_bar.clone());

    Self {
      progress: overall_bar,
      items: size,
      completed: AtomicU64::new(0),
      completed_index: AtomicU64::new(0),
      completed_items
    }
  }

  fn create_completed_progress_bar() -> ProgressBar {
    let completed_bar_style =
      ProgressStyle::with_template("{prefix}").unwrap();

    ProgressBar::new(1)
    .with_style(completed_bar_style)
    .with_finish(indicatif::ProgressFinish::Abandon)
  }

  pub async fn monitor(mut self, mut rx: Receiver<FileStatus>, start_time: Instant) -> R<()> {
    self.progress.tick();
    let timer_handle = {
      let pb = self.progress.clone();
      let h = thread::spawn(move || {
        while !pb.is_finished() {
          let current_time = Instant::now();
          let duration = current_time.duration_since(start_time);
          let millis = duration.as_millis();
          let seconds = duration.as_secs();
          let minutes = seconds / 60;
          let hours = minutes / 60;
          pb.set_message(format!("{:02}:{:02}:{:02}.{:02}", hours, minutes, seconds, millis));
          thread::sleep(Duration::from_millis(250));
        }
      });
      h
    };

    while let Ok(value) = rx.recv().await {
      match value {
        FileStatus::Success(pb) => {
          let completed = self.completed.get_mut();
          *completed += 1;
          self.progress.inc(1);
          self.progress.set_prefix(format!("{}/{}", completed, self.items));

          // If all items are completed, then finish
          if *completed >= self.items {
            self.progress.finish()
          }

          self.insert_completed_bar("file")
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

  fn insert_completed_bar(&mut self, arg: &str) {
    let current_index = self.completed_index.load(atomic::Ordering::Relaxed);
    if let Some(pb) = self.completed_items.get(current_index as usize) {
      pb.set_prefix(arg.to_owned());
      let next_index = self.completed_index.get_mut();
      *next_index += 1;
    }
  }
}

