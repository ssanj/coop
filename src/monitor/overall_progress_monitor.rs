use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tokio::sync::mpsc::Receiver;

use crate::model::{size_pretty, CopyError, FailedReason, FileName, FileSize, FileStatus, R};

struct State {
  completed: u64,
  completed_index: u64,
  completed_bytes: u64,
  inprogress_bytes: u64,
  completed_item_bars: Vec<ProgressBar>,
}

pub struct NumFiles(u64);

impl NumFiles {
  pub fn new(num_files: u64) -> Self {
    Self(num_files)
  }
}

pub struct TotalFileSize(u64);

impl TotalFileSize {
  pub fn new(total_size: u64) -> Self {
    Self(total_size)
  }
}

/// Monitors the overall progress of all file copies in progress
pub struct OverallProgressMonitor {
  progress: ProgressBar,
  items: u64,
  total_bytes: u64,
  state: Arc<Mutex<State>>,
}

impl OverallProgressMonitor {

  pub fn new(multi: &MultiProgress, num_files: NumFiles, total_file_size: TotalFileSize) -> Self {
    let completed_item_bars =
      (0..num_files.0)
      .map(|_| {
        let pb = Self::create_completed_progress_bar();
        multi.add(pb.clone())
      })
      .collect();

    let overall_bar =
      ProgressStyle::with_template("[{msg}] overall progress: {prefix} [{wide_bar:.green}]").unwrap();

    let overall_bar =
      ProgressBar::new(num_files.0)
      .with_style(overall_bar)
      .with_finish(indicatif::ProgressFinish::Abandon);

    // Add this at the end
    multi.add(overall_bar.clone());

    let state =
      Arc::new(
        Mutex::new(
          State {
            completed: 0,
            completed_index: 0,
            completed_bytes: 0,
            inprogress_bytes: 0,
            completed_item_bars,
          }
        )
      );

    Self {
      progress: overall_bar,
      items: num_files.0,
      total_bytes: total_file_size.0,
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

  /// This is a low cardinality event receiver.
  pub async fn monitor(self, mut rx: Receiver<FileStatus>, start_time: Instant) -> R<()> {
    self.progress.set_prefix(format!("0KB {}/{} (0/0)", 0, self.items));
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

    let inprogress_handle = {
      let pb = self.progress.clone();
      let state = self.state.clone();
      thread::spawn(move || {
        while !pb.is_finished() {
          let guard = state.lock().unwrap();
          Self::set_progress(&guard, &pb, self.items, self.total_bytes);
          drop(guard);
          thread::sleep(Duration::from_millis(250));
        }
      })
    };


    while let Some(value) = rx.recv().await {
      match value {
        FileStatus::Success(file_name, file_size, _) => {
          self.handle_succeeded(file_name, file_size)
        },

        FileStatus::Failed(FailedReason::ReadFailed(file_name, error, _)) => self.handle_failed(file_name, error),
        FileStatus::Failed(FailedReason::WriteFailed(file_name, error, _)) => self.handle_failed(file_name, error),
        FileStatus::Failed(FailedReason::FlushFailed(file_name, error, _)) => self.handle_failed(file_name, error),
        FileStatus::Failed(FailedReason::CouldNotReadSourceFile(file_name, error, _)) => self.handle_failed(file_name, error),
        FileStatus::Failed(FailedReason::CouldNotGetFileSize(file_name, error, _, _)) => self.handle_failed(file_name, error),
        FileStatus::Failed(FailedReason::CouldNotCreateDestinationFile(file_name, error, _)) => self.handle_failed(file_name, error),
        FileStatus::Failed(FailedReason::CouldNotCreateDestinationDir(file_name, error, _)) => self.handle_failed(file_name, error),
        FileStatus::Failed(FailedReason::FileSizesAreDifferent(file_name, _, _)) => self.handle_failed(file_name, CopyError::new("File sizes are different")),
        FileStatus::InProgress(bytes) => self.handle_inprogress(bytes),
        _ => ()
     }
    }

    let _ = timer_handle.join();
    let _ = inprogress_handle.join();

    Ok(())
  }


  fn handle_succeeded(&self, file: FileName, file_size: FileSize) {
    self.handle_end_state(Some(file_size.clone()), |state| Self::insert_completed_bar(&file.name(), file_size, state))
  }

  fn handle_failed(&self, file: FileName, error: CopyError) {
    self.handle_end_state(None, |state| Self::insert_failed_bar(&file.name(), &error.error(), state))
  }

  fn insert_completed_bar(arg: &str, file_size: FileSize, state: &mut MutexGuard<State>) {
    Self::insert_bar(format!("{arg} ({file_size}) ✅"), state)
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

  fn handle_end_state<F: FnOnce(&mut MutexGuard<State>)>(&self, maybe_file_size: Option<FileSize>, update_completed_display: F) {
    let mut state_guard = self.state.lock().unwrap();
    state_guard.completed += 1;
    self.progress.inc(1);
    if let Some(file_size) = maybe_file_size {
      state_guard.completed_bytes += file_size.size()
    }
    self.set_progress_(&state_guard);

    // If all items are completed, then finish
    if state_guard.completed >= self.items {
      self.progress.finish();
    }

    update_completed_display(&mut state_guard)
  }

  fn handle_inprogress(&self, bytes: u64) {
    let mut state_guard = self.state.lock().unwrap();
    state_guard.inprogress_bytes += bytes;
    drop(state_guard)
  }

  fn set_progress_(&self, state_guard: &MutexGuard<State>) {
    Self::set_progress(state_guard, &self.progress, self.items, self.total_bytes)
  }

  fn set_progress(state_guard: &MutexGuard<State>, pb: &ProgressBar, items: u64, total_bytes: u64) {
    pb.set_prefix(format!("{} {}/{} ({}/{})", size_pretty(state_guard.inprogress_bytes), state_guard.completed, items, size_pretty(state_guard.completed_bytes), size_pretty(total_bytes)));
  }
}

