use std::path::PathBuf;

use clap::Parser;

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
}

pub fn get_cli_args() -> Args {
  Args::parse()
}
