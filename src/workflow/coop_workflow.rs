use std::cmp::{max, min};

use regex::Regex;
use walkdir::{DirEntry, WalkDir};
use indicatif::MultiProgress;
use once_cell::sync::Lazy;
use tokio::sync::mpsc::{self};
use tokio::task::JoinSet;

use crate::cli::Args;
use crate::copy::FileCopy;
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

    let files_to_copy = Self::get_source_files(source_dir, ignored_regexes);

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

  fn ignored(ignored_regexes: &[Regex], de: &DirEntry) -> bool {
    ignored_regexes
      .into_iter()
      .any(|r| r.is_match(de.path().to_string_lossy().as_ref()))
  }

  fn get_source_files(source_dir: &std::path::PathBuf, ignored_regexes: &Vec<Regex>) -> Vec<String> {
    WalkDir::new(source_dir)
      .into_iter()
      .filter_map(|de| {
        de
          .ok()
          .filter(|d| {
            // We only want files and not directories or symlinks
            // We might want to filter out certain files like .DS_Store
            d.file_type().is_file() && !Self::ignored(ignored_regexes, d)
          })
          .and_then(|f| {
            f
              .path()
              .strip_prefix(source_dir)
              .ok()
              .map(|f1| f1.to_string_lossy().to_string() )
          })
      })
      .collect()
  }
}
