use std::str::FromStr;
use regex::Regex;
use once_cell::sync::Lazy;
use std::cmp::min;

#[derive(Debug, PartialEq)]
pub enum BufferUnit {
  KB,
  MB
}

#[derive(Debug, PartialEq)]
pub struct Buffer(u16, BufferUnit);

pub static BUFFER_REG: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\d+)((KB|MB))$").unwrap());

#[derive(Debug, PartialEq)]
pub struct BufferError(String);

impl BufferError {

  const FORMAT: &'static str = "Expected format: <num><unit>, where num = <number>, unit = <KB|MB>";

  pub fn format_error(arg: &str) -> Self {
    BufferError(format!("Invalid buffer format supplied: '{}'. {}", arg, Self::FORMAT))
  }

  pub fn size_error(error: String) -> Self {
    BufferError(format!("Invalid buffer size supplied: {}. {}", error, Self::FORMAT))
  }
}

impl FromStr for Buffer {
  type Err = BufferError;

  fn from_str(arg: &str) -> Result<Self, Self::Err> {
    match BUFFER_REG.captures(arg) {
      Some(matches) => {
        if matches.len() < 2 {
          return Err(BufferError::format_error(arg))
        }

        let incoming_size = &matches[1];
        let unit = &matches[2];

        let size =
          incoming_size
            .parse::<u16>()
            .map_err(|e| BufferError::size_error(e.to_string()))?;

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

        Ok(Buffer(checked_size, buffer_unit))

      },
      None => Err(BufferError::format_error(arg))
    }
  }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn succeeds_on_valid_kb_buffer_size() {
      let buffer: Result<Buffer, BufferError> = FromStr::from_str("1KB");

      assert_eq!(buffer, Ok(Buffer(1, BufferUnit::KB)))
    }

    #[test]
    fn succeeds_on_max_kb_buffer_size() {
      let buffer: Result<Buffer, BufferError> = FromStr::from_str("1024KB");

      assert_eq!(buffer, Ok(Buffer(1024, BufferUnit::KB)))
    }

    #[test]
    fn truncates_on_over_max_kb_buffer_size() {
      let buffer: Result<Buffer, BufferError> = FromStr::from_str("1025KB");
      assert_eq!(buffer, Ok(Buffer(1024, BufferUnit::KB)))
    }

    #[test]
    fn succeeds_on_valid_mb_buffer_size() {
      let buffer: Result<Buffer, BufferError> = FromStr::from_str("1MB");

      assert_eq!(buffer, Ok(Buffer(1, BufferUnit::MB)))
    }

    #[test]
    fn succeeds_on_max_mb_buffer_size() {
      let buffer: Result<Buffer, BufferError> = FromStr::from_str("128MB");

      assert_eq!(buffer, Ok(Buffer(128, BufferUnit::MB)))
    }

    #[test]
    fn truncates_on_over_max_mb_buffer_size() {
      let buffer: Result<Buffer, BufferError> = FromStr::from_str("256MB");

      assert_eq!(buffer, Ok(Buffer(128, BufferUnit::MB)))
    }

    #[test]
    fn fails_with_invalid_size() {
      let buffer: Result<Buffer, BufferError> = FromStr::from_str("OneMB");

      assert_eq!(buffer, Err(BufferError("Invalid buffer format supplied: 'OneMB'. Expected format: <num><unit>, where num = <number>, unit = <KB|MB>".to_owned())))
    }

    #[test]
    fn fails_with_invalid_unit() {
      let buffer: Result<Buffer, BufferError> = FromStr::from_str("1GB");
      assert_eq!(buffer, Err(BufferError("Invalid buffer format supplied: '1GB'. Expected format: <num><unit>, where num = <number>, unit = <KB|MB>".to_owned())))
    }

    #[test]
    fn fails_with_invalid_input_start() {
      let buffer: Result<Buffer, BufferError> = FromStr::from_str(" 1KB");
      assert_eq!(buffer, Err(BufferError("Invalid buffer format supplied: ' 1KB'. Expected format: <num><unit>, where num = <number>, unit = <KB|MB>".to_owned())))
    }

    #[test]
    fn fails_with_invalid_input_end() {
      let buffer: Result<Buffer, BufferError> = FromStr::from_str("1KB ");
      assert_eq!(buffer, Err(BufferError("Invalid buffer format supplied: '1KB '. Expected format: <num><unit>, where num = <number>, unit = <KB|MB>".to_owned())))
    }
}
