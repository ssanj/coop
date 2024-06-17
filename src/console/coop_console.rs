use dialoguer::{theme::ColorfulTheme, FuzzySelect};
use console::style;
use crate::{args::BufferSize, copy::SourceFile};

pub struct CoopConsole;

pub enum UserResult {
  Continue,
  Cancel,
  Error(String)
}

const KB: u64 = 1024;
const MB: u64 = 1048576;
const GB: u64 = 1073741824;

impl CoopConsole {
  pub fn show_copy_state(files_to_copy: &[SourceFile], concurrency: u8, buffer_size: &BufferSize, destination_dir: &str) -> UserResult {
    let files: Vec<(String, u64)> =
      files_to_copy
        .iter()
        .map(|sf| (sf.relative_path().clone(), sf.size()))
        .collect();

    println!("{}:", style("Source files").green());
    for (index, (file, size)) in files.iter().enumerate() {
      println!("  {:06} - {} ({})", index + 1, style(file).cyan(), style(Self::size_pretty(*size)).yellow())
    }

    let total_size = files.iter().map(|(_, size)| *size).sum();
    println!("{}: {}", style("Concurrency").green(), concurrency);
    println!("{}: {}", style("Buffer size").green(), buffer_size);
    println!("{}: {}", style("Destination").green(), destination_dir);
    println!("{}: {}", style("Total size").green(), Self::size_pretty(total_size));

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

  fn size_pretty(size: u64) -> String {
    if size >= GB {
      format!("{:.2}GB", size as f64 / GB as f64)
    } else if size >= MB {
      format!("{:.2}MB", size as f64 / MB as f64)
    } else if size >= KB {
      format!("{:.2}KB", size as f64 / KB as f64)
    } else {
      format!("{:.2}B", size)
    }
  }
}
