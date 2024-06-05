use dialoguer::{theme::ColorfulTheme, FuzzySelect};
use console::style;
use crate::{args::BufferSize, copy::SourceFile};

pub struct CoopConsole;

pub enum UserResult {
  Continue,
  Cancel,
  Error(String)
}

impl CoopConsole {
  pub fn show_copy_state(files_to_copy: &[SourceFile], concurrency: u16, buffer_size: &BufferSize, destination_dir: &str) -> UserResult {
    let files: Vec<_> =
      files_to_copy
        .iter()
        .map(|sf| sf.relative_path().clone())
        .collect();

    println!("{}:", style("Source files").green());
    for (index, file) in files.iter().enumerate() {
      println!("  {:06} - {}", index + 1, style(file).cyan())
    }
    println!("{}: {}", style("Concurrency").green(), concurrency);
    println!("{}: {}", style("Buffer size").green(), buffer_size);
    println!("{}: {}", style("Destination").green(), destination_dir);

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
