use regex::Regex;
use walkdir::{DirEntry, WalkDir};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SourceFile {
  full: PathBuf,
  relative: PathBuf,
  size: u64,
  file_type: FileType
}

#[derive(Debug, Clone)]
enum FileType {
  File(u64),
  Dir
}

impl SourceFile {

  fn from_dir<P: AsRef<Path>>(source_dir: P, de: DirEntry, size: u64) -> Option<Self> {
    let full = de.path().to_owned();
    de.
      into_path()
      .strip_prefix(source_dir)
      .ok()
      .map(|rel| {
        Self {
          full,
          relative: rel.to_owned(),
          size,
          file_type: FileType::Dir
        }
      })
  }

  fn from_file<P: AsRef<Path>>(source_file: P, size: u64) -> Self {
    let full = source_file.as_ref().to_owned();
    // In case of a file, the relative path is the filename
    let relative = full.file_name().map(PathBuf::from).unwrap();
    let file_type = FileType::File(size);

    Self {
      full,
      relative,
      size,
      file_type
    }
  }

  /// File name
  pub fn file_name(&self) -> String {
    match self.file_type {
      FileType::File(_) => self.relative.to_string_lossy().into(),
      FileType::Dir => self.full.file_name().unwrap().to_string_lossy().into(),
    }
  }

  /// File name with subdirectories relative to the supplied source directory
  pub fn relative_path(&self) -> String {
    self.relative.to_string_lossy().into()
  }


  pub fn full_path(&self) -> &Path {
    self.full.as_path()
  }

  pub fn size(&self) -> u64 {
    self.size
  }

  pub fn get_source_files(source_dir: &PathBuf, ignored_regexes: &[Regex]) -> Vec<SourceFile> {
    let file_type =
      fs::File::open(source_dir)
        .and_then(|f| f.metadata() )
        .map(|m| {
            if m.is_file() {
              FileType::File(m.len())
            } else {
              FileType::Dir
            }
        })
        .unwrap_or(FileType::Dir);

    match file_type {
      FileType::File(size) => Self::get_file(source_dir, size),
      FileType::Dir => Self::get_directory_files(source_dir, ignored_regexes),
    }
  }

  fn ignored(ignored_regexes: &[Regex], de: &DirEntry) -> bool {
    ignored_regexes
      .iter()
      .any(|r| r.is_match(de.path().to_string_lossy().as_ref()))
  }

  fn get_file(source_file: &PathBuf, size: u64) -> Vec<SourceFile> {
    vec![SourceFile::from_file(source_file, size)]
  }

  fn get_directory_files(source_dir: &PathBuf, ignored_regexes: &[Regex]) -> Vec<SourceFile> {
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
          .and_then(|file| {
            file
              .metadata()
              .ok()
              .and_then(|meta| {
                SourceFile::from_dir(source_dir, file, meta.len())
              })
          })
      })
      .collect()
  }
}
