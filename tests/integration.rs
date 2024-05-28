use assert_cmd::Command;

use std::format as s;

#[test]
fn returns_version() -> Result<(), Box<dyn std::error::Error>> {
  let mut cmd = Command::cargo_bin("coop").unwrap();

  let version = env!("CARGO_PKG_VERSION");
  let expected_version_string = s!("coop {}\n", version);

  cmd
    .arg("-V")
    .assert()
    .success()
    .stdout(expected_version_string);

  Ok(())
}
