use regex::Regex;
use walkdir::{DirEntry, WalkDir};

use crate::cli::Args;

pub struct CoopWorkflow {
  args: Args
}

impl CoopWorkflow {

  pub fn new(args: Args) -> Self {
    Self {
      args
    }
  }

  pub fn run(self) {
    let args = self.args;
    let source_dir = &args.source_dir;
    let ignored_regexes = &args.ignore;

    let files_to_copy: Vec<_> =
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
        .collect();

    println!("{:?}", files_to_copy)
  }

  fn ignored(ignored_regexes: &[Regex], de: &DirEntry) -> bool {
    ignored_regexes
      .into_iter()
      .any(|r| r.is_match(de.path().to_string_lossy().as_ref()))
  }
}
