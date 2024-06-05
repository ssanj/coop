use indicatif::{ProgressBar, ProgressStyle, MultiProgress};

#[derive(Debug, Clone)]
pub struct MyProgressBar {
  primary: ProgressBar,
  secondary: ProgressBar,
  error: ProgressBar,
  separator: ProgressBar,
}

impl MyProgressBar {
  pub fn new(multi: &MultiProgress) -> Self {
    let primary_style = ProgressStyle::with_template("[{elapsed_precise}] [{wide_bar:.green}] {bytes:.green}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap();
    let primary =
      ProgressBar::new(0)
      .with_style(primary_style)
      .with_finish(indicatif::ProgressFinish::AndClear);

    let secondary_style = ProgressStyle::with_template("file: {prefix}, status: {msg}").unwrap();
    let secondary =
      ProgressBar::new(10)
        .with_style(secondary_style)
        .with_finish(indicatif::ProgressFinish::AndClear);

    let error_style = ProgressStyle::with_template("{msg:.red}").unwrap();
    let error =
      ProgressBar::new(10)
        .with_style(error_style)
        .with_finish(indicatif::ProgressFinish::AndClear);

    let separator =
      ProgressBar::new(10)
        .with_style(ProgressStyle::with_template("{wide_bar:.blue}").unwrap().progress_chars("--"))
        .with_finish(indicatif::ProgressFinish::AndClear);

    multi.add(primary.clone());
    multi.add(secondary.clone());
    multi.add(error.clone());
    multi.add(separator.clone());

    Self {
      primary,
      secondary,
      error,
      separator
    }
  }

  pub fn set_file_size(&self, file_size: u64) {
    self.primary.set_length(file_size)
  }

  pub fn tick(&self) {
    self.primary.tick();
    self.secondary.tick();
    self.error.tick();
    self.separator.tick();
  }

  pub fn set_error(&self, msg: &str) {
    self.error.set_message(msg.to_owned());
    // When there is an error, consider this progress as finished.
    self.primary.finish()
  }

  pub fn set_status(&self, msg: &str) {
    self.secondary.set_message(msg.to_owned())
  }

  pub fn set_prefix(&self, prefix: String) {
    self.secondary.set_prefix(prefix)
  }

  pub fn update_progress(&self, bytes_written: u64) {
    self.primary.inc(bytes_written)
  }

  pub fn complete(&self, msg: &str) {
    self.secondary.set_message(msg.to_owned());
  }

  pub fn clear(&self) {
    self.primary.finish_and_clear();
    self.secondary.finish_and_clear();
    self.error.finish_and_clear();
    self.separator.finish_and_clear();
  }
}
