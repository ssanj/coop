use std::fs::FileType;

use walkdir::WalkDir;

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

    let files_to_copy: Vec<_> =
      WalkDir::new(source_dir)
        .into_iter()
        .filter_map(|de| {
          de
            .ok()
            .filter(|d| {
              // We only want files and not directories or symlinks
              // We might want to filter out certain files like .DS_Store
              d.file_type().is_file()
            })
            .map(|f| f.file_name().to_string_lossy().to_string() )
        })
        .collect();

    println!("{:?}", files_to_copy)
  }
}
