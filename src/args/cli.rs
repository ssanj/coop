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

   /// Source directory or file to copy files from
   #[arg(short, long)]
   pub source: PathBuf,

   /// Destination directory to copy files to
   #[arg(short, long)]
   pub destination_dir: PathBuf,

   /// The maximum number of file copies to perform concurrently (1-16).
   #[arg(short, long, default_value="4", value_parser=clap::value_parser!(u8).range(1..16))]
   pub concurrency: u8,

   /// The maximum buffer size to use when copying. Maximum of 1024KB or 128MB. [default: 1MB]
   #[arg(short, long, value_parser = clap::value_parser!(BufferSize))]
   pub buffer_size: Option<BufferSize>,

   /// Files to ignore during copy.
   ///
   /// Can be specified multiple times.
   /// Accepts a regular expression which filters the file path from the current directory.
   /// Only applies to sources that are directories; not files.
   ///
   /// Example: --ignore '.git'
   ///
   /// Note: When ignores are supplied the defaults are not used.
   #[arg(short, long, default_values=[".DS_Store", ".git", "/target"])]
   pub ignore: Vec<Regex>,

   /// Skip asking verification on copy
   #[arg(long)]
   pub skip_verify: bool
}

pub fn get_cli_args() -> Args {
  Args::parse()
}
