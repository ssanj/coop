use std::str::FromStr;
use clap::ValueEnum;
use regex::Regex;
use once_cell::sync::Lazy;
use std::cmp::min;
use std::fmt;

pub static BUFFER_REG: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\d+)((KB|MB))$").unwrap());

#[derive(Debug, PartialEq, Clone, ValueEnum)]
pub enum BufferUnit {
  KB,
  MB
}

#[derive(Debug, PartialEq, Clone)]
pub struct BufferSize(u16, BufferUnit);

impl BufferSize {

  // 1MB
  pub const DEFAULT_BUFFER_SIZE: BufferSize = BufferSize(1, BufferUnit::MB);

  pub fn value(&self) -> u64 {
    match self.1 {
      BufferUnit::KB => 1024 * self.0 as u64,
      BufferUnit::MB => 1048576 * self.0 as u64,
    }
  }
}

impl FromStr for BufferSize {
  type Err = String;

  fn from_str(arg: &str) -> Result<Self, Self::Err> {
    match BUFFER_REG.captures(arg) {
      Some(matches) => {
        if matches.len() < 2 {
          return Err(BufferSizeError::format_error(arg))
        }

        let incoming_size = &matches[1];
        let unit = &matches[2];

        let size =
          incoming_size
            .parse::<u16>()
            .map_err(|e| BufferSizeError::size_error(e.to_string()))?;

        let buffer_unit =
          if unit == "MB" {
            BufferUnit::MB
          } else {
            BufferUnit::KB
          };

        let checked_size =
          match buffer_unit {
            BufferUnit::KB => min(1024, size), // max one MB
            BufferUnit::MB => min(128, size), // max 128 MB
          };

        Ok(BufferSize(checked_size, buffer_unit))

      },
      None => Err(BufferSizeError::format_error(arg))
    }
  }
}

impl fmt::Display for BufferSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let unit = match self.1 {
          BufferUnit::KB => "KB",
          BufferUnit::MB => "MB",
        };
        write!(f, "{}{}", self.0, unit)
    }
}


#[derive(Debug, PartialEq)]
pub struct BufferSizeError(String);


impl BufferSizeError {

  const FORMAT: &'static str = "Expected format: <num><unit>, where num = <number>, unit = <KB|MB>, max KB is 1024, max MB is 128";

  pub fn format_error(arg: &str) -> String {
    format!("Invalid buffer format supplied: '{}'. {}", arg, Self::FORMAT)
  }

  pub fn size_error(error: String) -> String {
    format!("Invalid buffer size supplied: {}. {}", error, Self::FORMAT)
  }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn succeeds_on_valid_kb_buffer_size() {
      let buffer: Result<BufferSize, String> = FromStr::from_str("1KB");

      assert_eq!(buffer, Ok(BufferSize(1, BufferUnit::KB)))
    }

    #[test]
    fn succeeds_on_max_kb_buffer_size() {
      let buffer: Result<BufferSize, String> = FromStr::from_str("1024KB");

      assert_eq!(buffer, Ok(BufferSize(1024, BufferUnit::KB)))
    }

    #[test]
    fn truncates_on_over_max_kb_buffer_size() {
      let buffer: Result<BufferSize, String> = FromStr::from_str("1025KB");
      assert_eq!(buffer, Ok(BufferSize(1024, BufferUnit::KB)))
    }

    #[test]
    fn succeeds_on_valid_mb_buffer_size() {
      let buffer: Result<BufferSize, String> = FromStr::from_str("1MB");

      assert_eq!(buffer, Ok(BufferSize(1, BufferUnit::MB)))
    }

    #[test]
    fn succeeds_on_max_mb_buffer_size() {
      let buffer: Result<BufferSize, String> = FromStr::from_str("128MB");

      assert_eq!(buffer, Ok(BufferSize(128, BufferUnit::MB)))
    }

    #[test]
    fn truncates_on_over_max_mb_buffer_size() {
      let buffer: Result<BufferSize, String> = FromStr::from_str("256MB");

      assert_eq!(buffer, Ok(BufferSize(128, BufferUnit::MB)))
    }

    #[test]
    fn fails_with_invalid_size() {
      let buffer = <BufferSize as FromStr>::from_str("OneMB").unwrap_err();
      assert_eq!(buffer, "Invalid buffer format supplied: 'OneMB'. Expected format: <num><unit>, where num = <number>, unit = <KB|MB>, max KB is 1024, max MB is 128".to_owned())
    }

    #[test]
    fn fails_with_invalid_unit() {
      let buffer = <BufferSize as FromStr>::from_str("1GB").unwrap_err();
      assert_eq!(buffer, "Invalid buffer format supplied: '1GB'. Expected format: <num><unit>, where num = <number>, unit = <KB|MB>, max KB is 1024, max MB is 128".to_owned())
    }

    #[test]
    fn fails_with_invalid_input_start() {
      let buffer = <BufferSize as FromStr>::from_str(" 1KB").unwrap_err();
      assert_eq!(buffer, "Invalid buffer format supplied: ' 1KB'. Expected format: <num><unit>, where num = <number>, unit = <KB|MB>, max KB is 1024, max MB is 128".to_owned())
    }

    #[test]
    fn fails_with_invalid_input() {
      let buffer = <BufferSize as FromStr>::from_str("ABC").unwrap_err();
      assert_eq!(buffer, "Invalid buffer format supplied: 'ABC'. Expected format: <num><unit>, where num = <number>, unit = <KB|MB>, max KB is 1024, max MB is 128".to_owned())
    }

    #[test]
    fn fails_with_invalid_input_end() {
      let buffer = <BufferSize as FromStr>::from_str("1KB ").unwrap_err();
      assert_eq!(buffer, "Invalid buffer format supplied: '1KB '. Expected format: <num><unit>, where num = <number>, unit = <KB|MB>, max KB is 1024, max MB is 128".to_owned())
    }
}
