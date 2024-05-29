use regex::Regex;
use walkdir::{DirEntry, WalkDir};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SourceFile {
  full: PathBuf,
  relative: PathBuf
}

impl SourceFile {

  fn new<P: AsRef<Path>>(source_dir: P, de: DirEntry) -> Option<Self> {
    let full = de.path().to_owned();
    de.
      into_path()
      .strip_prefix(source_dir)
      .ok()
      .map(|rel| {
        Self {
          full,
          relative: rel.to_owned()
        }
      })
  }

  pub fn file_name(&self) -> String {
    self.full.file_name().unwrap().to_string_lossy().into()
  }

  pub fn relative_path(&self) -> String {
    self.relative.to_string_lossy().to_string()
  }

  pub fn full_path(&self) -> &Path {
    self.full.as_path()
  }

  pub fn get_source_files(source_dir: &PathBuf, ignored_regexes: &Vec<Regex>) -> Vec<SourceFile> {
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
          .and_then(|file| SourceFile::new(source_dir, file))
      })
      .collect()
  }

  fn ignored(ignored_regexes: &[Regex], de: &DirEntry) -> bool {
    ignored_regexes
      .into_iter()
      .any(|r| r.is_match(de.path().to_string_lossy().as_ref()))
  }
}
