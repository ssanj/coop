use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use chrono::TimeDelta;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tokio::sync::mpsc::Receiver;

use crate::model::{size_pretty, CoopError, CopyError, FailedReason, FileName, FileSize, FileStatus, R};

struct State {
  completed: u64,
  completed_bytes: u64,
  inprogress_bytes: u64,
  log: File,
  error_bar: ProgressBar,
  errors: Vec<String>,
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
  overall_bar: ProgressBar,
  stats_bar: ProgressBar,
  items: u64,
  total_bytes: u64,
  state: Arc<Mutex<State>>,
  start_time: Option<Instant>
}

impl OverallProgressMonitor {

  pub fn new(multi: &MultiProgress, num_files: NumFiles, total_file_size: TotalFileSize) -> Result<Self, CoopError> {
    let overall_bar_style =
      ProgressStyle::with_template("[{msg}] {prefix} [{wide_bar:.green}]").unwrap();

    let overall_bar =
      ProgressBar::new(num_files.0)
      .with_style(overall_bar_style)
      .with_finish(indicatif::ProgressFinish::Abandon);

    let stats_bar_style =
      ProgressStyle::with_template("{prefix}").unwrap();

    let stats_bar =
      ProgressBar::new(0)
        .with_style(stats_bar_style)
        .with_finish(indicatif::ProgressFinish::Abandon);

    let error_bar_style =
      ProgressStyle::with_template("{prefix}").unwrap();


    let error_bar =
      ProgressBar::new(num_files.0)
      .with_style(error_bar_style)
      .with_finish(indicatif::ProgressFinish::Abandon);

    multi.add(error_bar.clone());

    // Add this at the end
    multi.add(overall_bar.clone());
    multi.add(stats_bar.clone());

    // Remove the file if it exists
    let _ = std::fs::remove_file("coop.log");

    let log: File =
      OpenOptions::new()
        .create_new(true)
        .append(true)
        .open("coop.log")
        .map_err(|e| CoopError::CouldNotOpenLogFile(e.to_string()))?;

    let state =
      Arc::new(
        Mutex::new(
          State {
            completed: 0,
            completed_bytes: 0,
            inprogress_bytes: 0,
            log,
            error_bar,
            errors: vec![]
          }
        )
      );

    Ok(
      Self {
        overall_bar,
        stats_bar,
        items: num_files.0,
        total_bytes: total_file_size.0,
        state,
        start_time: None
      }
    )
  }


  /// This is a low cardinality event receiver.
  pub async fn monitor(mut self, mut rx: Receiver<FileStatus>, start_time: Instant) -> R<()> {
    self.start_time = Some(start_time); // Set the start time
    {
      let state_guard = self.state.lock().unwrap();
      self.set_progress(&state_guard);
      Self::set_stats(&state_guard, &self.stats_bar, self.total_bytes, start_time);
      drop(state_guard)
    }

    let timer_handle = {
      let pb = self.overall_bar.clone();
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
      let pb = self.stats_bar.clone();
      let state = self.state.clone();
      thread::spawn(move || {
        while !pb.is_finished() {
          let guard = state.lock().unwrap();
          Self::set_stats(&guard, &pb, self.total_bytes, start_time);
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
        FileStatus::Failed(FailedReason::CouldNotGetDestinationFileSize(file_name, error, _)) => self.handle_failed(file_name, error),
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
    Self::log(format!("{arg} ({file_size}) ✅"), state)
  }

  fn insert_failed_bar(arg: &str, error: &str, state: &mut MutexGuard<State>) {
    let errors = &mut state.errors;
    let error_string = format!("{arg} ({}) ❌", error);
    errors.push(error_string.clone());
    state.error_bar.clone().with_prefix(state.errors.join("\n"));
    Self::log(error_string, state)
  }

  fn log(arg: String, state: &mut MutexGuard<State>) {
    let mut log = &state.log;
    let buf = format!("{arg}\n");
    let _ = log.write(buf.as_bytes()).unwrap();
  }

  fn handle_end_state<F: FnOnce(&mut MutexGuard<State>)>(&self, maybe_file_size: Option<FileSize>, update_completed_display: F) {
    let mut state_guard = self.state.lock().unwrap();
    state_guard.completed += 1;
    self.overall_bar.inc(1);
    if let Some(file_size) = maybe_file_size {
      state_guard.completed_bytes += file_size.size()
    }
    self.set_progress(&state_guard);
    Self::set_stats(&state_guard, &self.stats_bar, self.total_bytes, self.start_time.unwrap());

    // If all items are completed, then finish
    if state_guard.completed >= self.items {
      self.overall_bar.finish();
      self.stats_bar.finish();
    }

    update_completed_display(&mut state_guard)
  }

  fn handle_inprogress(&self, bytes: u64) {
    let mut state_guard = self.state.lock().unwrap();
    state_guard.inprogress_bytes += bytes;
    drop(state_guard)
  }

  fn set_progress(&self, state_guard: &MutexGuard<State>) {
    self.overall_bar.set_prefix(format!("completed:{}/{}", state_guard.completed, self.items));
  }

  fn set_stats(state_guard: &MutexGuard<State>, pb: &ProgressBar, total_bytes: u64, start_time: Instant) {
    let elaped_time_seconds = start_time.elapsed().as_secs();

    let speed =
      if elaped_time_seconds == 0 {
        0
      } else {
        state_guard.inprogress_bytes / elaped_time_seconds
      };

    let remaining_bytes = total_bytes - state_guard.inprogress_bytes;
    let estimated_completion_time =
      if speed > 0 {
        let seconds_remaining = remaining_bytes / speed; // bytes/second
        use chrono::prelude::*;

        let local_now = Local::now();
        let delta = TimeDelta::from_std(Duration::from_secs(seconds_remaining)).unwrap();
        let estimated_completion_time = local_now + delta;
        estimated_completion_time.format("%I:%M:%S%p").to_string()
      } else {
        "00:00:00--".to_owned()
      };

      let duration =
        if speed > 0 {
          let seconds_remaining = remaining_bytes / speed; // bytes/second
          let current = Instant::now();
          let end_time = current.checked_add(Duration::from_secs(seconds_remaining)).unwrap();

          let estimated_duration = end_time.duration_since(start_time);
          let seconds = estimated_duration.as_secs();
          let minutes = seconds / 60;
          let hours = minutes / 60;
          if hours > 0 {
            format!("{:02}h {:02}m {:02}s", hours, minutes % 60, seconds % 60)
          } else if minutes > 0 {
            format!("{:02}m {:02}s", minutes % 60, seconds % 60)
          } else {
            format!("{:02}s", seconds % 60)
          }
        } else {
          format!("{:^11}", "00h 00m 00s".to_owned())
        };

    pb.set_prefix(
      format!(
        "copied:{} files:({}/{}) speed:({}) done:({}) takes:({})",
        size_pretty(state_guard.inprogress_bytes),
        size_pretty(state_guard.completed_bytes),
        size_pretty(total_bytes),
        size_pretty(speed),
        estimated_completion_time,
        duration
      )
    );
  }
}

