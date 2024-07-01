use std::time::Instant;
use std::cmp::{max, min};

use indicatif::MultiProgress;
use tokio::sync::mpsc::{self};
use tokio::task::JoinSet;

use crate::args::BufferSize;
use crate::cli::Args;
use crate::console::{CoopConsole, UserResult};
use crate::copy::{FileCopy, SourceFile};
use crate::model::{FileStatus, InProgress};
use crate::monitor::{
  FileInProgressMonitor, InProgressSender, LifecycleEventMonitor, LifecycleEventSender, MonitorMux, NumFiles, OverallProgressMonitor, OverallProgressSender, TotalFileSize
};

pub struct CoopWorkflow {
  args: Args
}

impl CoopWorkflow {

  pub fn new(args: Args) -> Self {
    Self {
      args
    }
  }

  pub async fn run(self) {
    let args = self.args;
    let source = &args.source;
    let destination_dir = &args.destination_dir;
    let ignored_regexes = &args.ignore;
    let concurrency = args.concurrency;
    let buffer_size = args.buffer_size.unwrap_or(BufferSize::DEFAULT_BUFFER_SIZE);
    let skip_verification = args.skip_verify;

    let files_to_copy = SourceFile::get_source_files(source, ignored_regexes);

    let total_file_sizes: u64 =
      files_to_copy
        .iter()
        .map(|sf| sf.size())
        .sum();

    if !skip_verification {
      let selection =
        CoopConsole::show_copy_state(
          &files_to_copy,
          concurrency,
          &buffer_size,
          destination_dir.to_str().unwrap_or("<Unknown>"),
          total_file_sizes
        );

      match selection {
        UserResult::Continue => (),
        _ => return
      };
    }

    let multi = MultiProgress::new();

    // This should fix the flickering after 0.17.8+
    multi.set_move_cursor(true);

    let copy_tasks: Vec<_> =
      files_to_copy
        .into_iter()
        .map(|f| FileCopy::new(f, destination_dir, &multi) )
        .collect();

    // For low cardinality events
    let (lifecycle_event_sender, lifecycle_event_receiver) = mpsc::channel::<FileStatus>(1000);
    let (overall_progress_sender, overall_progress_receiver) = mpsc::channel::<FileStatus>(1000);

    // For high cardinality events.
    // Allow up to 100,000 progress messages before blocking
    // Depending on the buffer size, the number of progress updates/s can be huge
    let (inprogress_sender, inprogress_receiver) = mpsc::channel::<InProgress>(100000);

    let lifecycle_event_monitor_fut = LifecycleEventMonitor::monitor(lifecycle_event_receiver);

    let overall_monitor =
      OverallProgressMonitor::new(&multi, NumFiles::new(copy_tasks.len() as u64), TotalFileSize::new(total_file_sizes))
        .unwrap_or_else(|e| panic!("{e}"));

    let overall_monitor_fut = overall_monitor.monitor(overall_progress_receiver, Instant::now());

    let progress_monitor_fut = FileInProgressMonitor::monitor(inprogress_receiver);

    let mut join_set = JoinSet::new();
    // Start the monitors first, so we don't miss any messages
    join_set.spawn(lifecycle_event_monitor_fut);
    join_set.spawn(overall_monitor_fut);
    join_set.spawn(progress_monitor_fut);

    let mut running = 0_u8;
    for task in copy_tasks {
      join_set.spawn(
        task.copy(
          buffer_size.clone(),
          MonitorMux::new(
            InProgressSender::new(inprogress_sender.clone()),
            LifecycleEventSender::new(lifecycle_event_sender.clone()),
            OverallProgressSender::new(overall_progress_sender.clone())
          )
        )
      );
      running = min(running + 1, concurrency);

      if running >= concurrency {
        // Wait for a single task to complete so we fall below the concurrency threshold
        // This would only wait for file copy tasks, as the monitors will stay alive until the last sender is dropped
        let _ = join_set.join_next().await;
        running = max(running - 1, 0);
      }
    };

    // Drop senders so the execution can complete
    drop(inprogress_sender);
    drop(lifecycle_event_sender);
    drop(overall_progress_sender);

    // Wait for any running tasks to complete
    while join_set.join_next().await.is_some() {}

    println!("See coop.log for the file list")
  }
}
