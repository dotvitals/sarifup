use std::io::Write;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;

const SARIFUP_PATH: &str = "target/debug/sarifup";

fn create_sarifup() -> Child {
    Command::new(SARIFUP_PATH)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg("tests/test.sarif")
        .spawn()
        .unwrap()
}

const VALID_SARIF_STR: &[u8] =
    b"{\"runs\":[{\"tool\":{\"driver\":{\"name\":\"test\"}}}],\"version\":\"1.0.0\"}";

#[test]
fn succeeds_and_returns_input_for_valid_sarif() {
    let mut sarifup = create_sarifup();
    let stdin = sarifup.stdin.as_mut().unwrap();

    stdin.write_all(VALID_SARIF_STR).unwrap();

    let output = sarifup.wait_with_output().unwrap();
    assert!(output.status.success());
    assert_eq!(&output.stdout, VALID_SARIF_STR);
}

#[test]
fn errors_with_message_when_unable_to_deserialize_stdin_sarif() {
    let mut sarifup = create_sarifup();
    let stdin = sarifup.stdin.as_mut().unwrap();

    let missing_ver_sarif_str = b"{\"runs\":[{\"tool\":{\"driver\":{\"name\":\"test\"}}}]}";
    stdin.write_all(missing_ver_sarif_str).unwrap();

    let output = sarifup.wait_with_output().unwrap();
    let error_message = std::str::from_utf8(&output.stderr).unwrap();
    assert!(!output.status.success());
    assert!(error_message.contains("Error deserializing stdin SARIF"));
}

#[test]
fn erorrs_with_message_when_cannot_get_filename_arg() {
    let mut sarifup = Command::new(SARIFUP_PATH)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stdin = sarifup.stdin.as_mut().unwrap();

    stdin.write_all(VALID_SARIF_STR).unwrap();

    let output = sarifup.wait_with_output().unwrap();
    let error_message = std::str::from_utf8(&output.stderr).unwrap();
    assert!(!output.status.success());
    assert!(error_message.contains("Error getting SARIF filename argument"));
}

#[test]
fn erorrs_with_message_when_file_cant_open() {
    let mut sarifup = Command::new(SARIFUP_PATH)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg("badfilename.name")
        .spawn()
        .expect("Failed to spawn app!");

    let stdin = sarifup.stdin.as_mut().unwrap();

    stdin.write_all(VALID_SARIF_STR).unwrap();

    let output = sarifup.wait_with_output().unwrap();
    let error_message = std::str::from_utf8(&output.stderr).unwrap();
    assert!(!output.status.success());
    assert!(error_message.contains("Error opening SARIF file"));
}

#[test]
fn errors_with_message_when_cannot_deserialize_sarif_file() {
    let mut sarifup = Command::new(SARIFUP_PATH)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg("tests/invalid.sarif")
        .spawn()
        .expect("Failed to spawn app!");

    let stdin = sarifup.stdin.as_mut().unwrap();
    stdin.write_all(VALID_SARIF_STR).unwrap();

    let output = sarifup.wait_with_output().unwrap();
    let error_message = std::str::from_utf8(&output.stderr).unwrap();
    assert!(!output.status.success());
    assert!(error_message.contains("Error deserializing SARIF file"));
}
