use std::path::PathBuf;
use regex::Regex;

use clap::Parser;

use super::buffer_size::BufferSize;

/// Making progress on your network file copy
#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
pub struct Args {
   /// Verbose debug logging
   #[arg(long)]
   pub verbose: bool,

   /// Source directory to copy files from
   #[arg(short,long)]
   pub source_dir: PathBuf,

   /// Destination directory to copy files to
   #[arg(short,long)]
   pub destination_dir: PathBuf,

   /// The maximum number of file copies to perform concurrently
   #[arg(short,long, default_value="4")]
   pub concurrency: u8,

   /// The maximum buffer size to use when copying. [default: 1MB]
   #[arg(short,long, value_parser = clap::value_parser!(BufferSize))]
   pub buffer_size: Option<BufferSize>,

   /// Files to ignore during copy.
   ///
   /// Can be specified multiple times.
   /// Accepts a regular expression which filters the file path from the current directory.
   ///
   /// Example: --ignore '.git'
   ///
   /// Note: When ignores are supplied the defaults are not used.
   #[arg(short,long, default_values=[".DS_Store", ".git", "/target"])]
   pub ignore: Vec<Regex>,
}

pub fn get_cli_args() -> Args {
  Args::parse()
}
