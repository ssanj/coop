use std::cmp::{max, min};

use indicatif::MultiProgress;
use once_cell::sync::Lazy;
use tokio::sync::mpsc::{self};
use tokio::task::JoinSet;

use crate::cli::Args;
use crate::console::{CoopConsole, UserResult};
use crate::copy::{FileCopy, SourceFile};
use crate::model::FileStatus;
use crate::monitor::FileCopyProgressMonitor;

static MULTI: Lazy<MultiProgress> = Lazy::new(|| MultiProgress::new());

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
    let source_dir = &args.source_dir;
    let destination_dir = &args.destination_dir;
    let ignored_regexes = &args.ignore;
    let concurrency = args.concurrency as u16;

    let files_to_copy = SourceFile::get_source_files(source_dir, ignored_regexes);
    let selection = CoopConsole::show_copy_state(&files_to_copy, concurrency, destination_dir.to_str().unwrap_or("<Unknown>"));

    let _ = match selection {
      UserResult::Continue => (),
      _ => return
    };

    let copy_tasks: Vec<_> =
      files_to_copy
        .into_iter()
        .map(|f| FileCopy::new(f, destination_dir, &MULTI) )
        .collect();

    let (tx, rx) = mpsc::channel::<FileStatus>(100);
    let monitor_fut = FileCopyProgressMonitor::monitor(rx);

    let mut join_set = JoinSet::new();
    // Start the monitor first, so we don't miss any messages
    join_set.spawn(monitor_fut);

    let mut running = 0_u16;
    for task in copy_tasks {
      join_set.spawn(task.copy(tx.clone())); // each task gets a copy of tx
      running = min(running + 1, u16::MAX);

      if running >= concurrency {
        // wait for a single task to complete so we fall below the concurrency threshold
        let _ = join_set.join_next().await;
        running = max(running - 1, 0);
      }
    };

    // This is an extra tx, drop it so the execution can complete
    drop(tx);

    // Wait for any running tasks to complete
    while let Some(_) = join_set.join_next().await {}
  }
}
