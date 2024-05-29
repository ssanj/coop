use regex::Regex;
use walkdir::{DirEntry, WalkDir};
use std::path::PathBuf;

pub struct SourceFile;

impl SourceFile {
  pub fn get_source_files(source_dir: &std::path::PathBuf, ignored_regexes: &Vec<Regex>) -> Vec<String> {
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

  fn ignored(ignored_regexes: &[Regex], de: &DirEntry) -> bool {
    ignored_regexes
      .into_iter()
      .any(|r| r.is_match(de.path().to_string_lossy().as_ref()))
  }
}
