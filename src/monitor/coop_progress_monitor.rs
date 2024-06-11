use std::sync::{Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tokio::sync::broadcast::Receiver;

use crate::model::{CopyError, FailedReason, FileName, FileStatus, R};

struct State {
  completed: u64,
  completed_index: u64,
  completed_item_bars: Vec<ProgressBar>,
}

pub struct CoopProgressMonitor {
  progress: ProgressBar,
  items: u64,
  state: Mutex<State>,
}

impl CoopProgressMonitor {

  pub fn new(multi: &MultiProgress, size: u64) -> Self {
    let completed_item_bars =
      (0..size)
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

    let state =
      Mutex::new(
        State {
          completed: 0,
          completed_index: 0,
          completed_item_bars,
        }
      );

    Self {
      progress: overall_bar,
      items: size,
      state
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
    self.progress.set_prefix(format!("{}/{}", 0, self.items));
    let timer_handle = {
      let pb = self.progress.clone();
      thread::spawn(move || {
        while !pb.is_finished() {
          let current_time = Instant::now();
          let duration = current_time.duration_since(start_time);
          let millis = duration.as_millis();
          let seconds = duration.as_secs();
          let minutes = seconds / 60;
          let hours = minutes / 60;
          pb.set_message(format!("{:02}:{:02}:{:02}.{:03}", hours, minutes % 60, seconds % 60, millis % 1000));
          thread::sleep(Duration::from_millis(250));
        }
      })
    };

    while let Ok(value) = rx.recv().await {
      match value {
        FileStatus::Success(file_name, _) => {
          self.handle_succeeded(file_name)
        },

        FileStatus::Failed(FailedReason::ReadFailed(file_name, error, _)) => self.handle_failed(file_name, error),
        FileStatus::Failed(FailedReason::WriteFailed(file_name, error, _)) => self.handle_failed(file_name, error),
        FileStatus::Failed(FailedReason::FlushFailed(file_name, error, _)) => self.handle_failed(file_name, error),
        FileStatus::Failed(FailedReason::CouldNotReadSourceFile(file_name, error, _)) => self.handle_failed(file_name, error),
        FileStatus::Failed(FailedReason::CouldNotGetFileSize(file_name, error, _, _)) => self.handle_failed(file_name, error),
        FileStatus::Failed(FailedReason::CouldNotCreateDestinationFile(file_name, error, _)) => self.handle_failed(file_name, error),
        FileStatus::Failed(FailedReason::CouldNotCreateDestinationDir(file_name, error, _)) => self.handle_failed(file_name, error),
        FileStatus::Failed(FailedReason::FileSizesAreDifferent(file_name, _, _)) => self.handle_failed(file_name, CopyError::new("File sizes are different")),

        _ => ()
     }
    }

    let _ = timer_handle.join();

    Ok(())
  }


  fn handle_succeeded(&mut self, file: FileName) {
    self.handle_end_state(|state| Self::insert_completed_bar(&file.name(), state))
  }

  fn handle_failed(&mut self, file: FileName, error: CopyError) {
    self.handle_end_state(|state| Self::insert_failed_bar(&file.name(), &error.error(), state))
  }

  fn insert_completed_bar(arg: &str, state: &mut MutexGuard<State>) {
    Self::insert_bar(format!("{arg} ✅"), state)
  }

  fn insert_failed_bar(arg: &str, error: &str, state: &mut MutexGuard<State>) {
    Self::insert_bar(format!("{arg} ({}) ❌", error), state)
  }

  fn insert_bar(arg: String, state: &mut MutexGuard<State>) {
    let current_index = state.completed_index;
    if let Some(pb) = state.completed_item_bars.get(current_index as usize) {
      pb.set_prefix(arg);
      state.completed_index += 1;
    }
  }

  fn handle_end_state<F: FnOnce(&mut MutexGuard<State>)>(&mut self, update_completed_display: F) {
    let mut state_guard = self.state.lock().unwrap();
    state_guard.completed += 1;
    self.progress.inc(1);
    self.progress.set_prefix(format!("{}/{}", state_guard.completed, self.items));

    // If all items are completed, then finish
    if state_guard.completed >= self.items {
      self.progress.finish();
    }

    update_completed_display(&mut state_guard)
  }

}

