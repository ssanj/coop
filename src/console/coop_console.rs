use dialoguer::{theme::ColorfulTheme, FuzzySelect};
use console::style;
use crate::{args::BufferSize, copy::SourceFile};
use crate::model::size_pretty;

pub struct CoopConsole;

pub enum UserResult {
  Continue,
  Cancel,
  Error(String)
}


impl CoopConsole {
  pub fn show_copy_state(
    files_to_copy: &[SourceFile],
    concurrency: u8,
    buffer_size: &BufferSize,
    destination_dir: &str,
    total_file_size: u64
  ) -> UserResult {
    let files: Vec<(String, u64)> =
      files_to_copy
        .iter()
        .map(|sf| (sf.relative_path().clone(), sf.size()))
        .take(50)
        .collect();

    println!("{}:", style("Source files").green());
    for (index, (file, size)) in files.iter().enumerate() {
      println!("  {:06} - {} ({})", index + 1, style(file).cyan(), style(size_pretty(*size)).yellow())
    }

    let num_files = files_to_copy.len();
    let displayed_num_files = files.len();

    if num_files > displayed_num_files {
      println!(" + ({})", num_files - displayed_num_files)
    }

    println!("{}: {}", style("Concurrency").green(), concurrency);
    println!("{}: {}", style("Buffer size").green(), buffer_size);
    println!("{}: {}", style("Destination").green(), destination_dir);
    println!("{}: {}", style("Files").green(), num_files);
    println!("{}: {}", style("Total size").green(), size_pretty(total_file_size));

    let options = ["no", "yes"];

    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
      .with_prompt("Continue with file copy?")
      .default(0)
      .items(&options)
      .interact()
      .map_err(|e| UserResult::Error(format!("Could not retrieve user options: {e}")))
      .and_then(|index| {
        options
          .get(index)
          .cloned()
          .map_or_else(
            || Err(UserResult::Error(format!("Invalid selection index: {index}"))),
            |v| {
              match v {
                "yes" => Ok(UserResult::Continue),
                _ => Ok(UserResult::Cancel),
              }
            })
        });

    selection.map_or_else(|e| e, |v| v)
  }

}
