pub const KB: u64 = 1024;
pub const MB: u64 = 1048576;
pub const GB: u64 = 1073741824;

pub fn size_pretty(size: u64) -> String {
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
