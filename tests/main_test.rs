use std::io::Write;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;

const VALID_SARIF_STR: &[u8] =
    b"{\"runs\":[{\"tool\":{\"driver\":{\"name\":\"test\"}}}],\"version\":\"1.0.0\"}";

fn create_sarifup() -> Child {
    Command::new("target/debug/sarifup")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .arg("test.json")
        .spawn()
        .expect("Failed to spawn app!")
}

#[test]
fn errors_with_invalid_json() {
    let mut sarifup = create_sarifup();
    let stdin = sarifup.stdin.as_mut().unwrap();

    stdin.write_all(b"d{").unwrap();

    let output = sarifup.wait_with_output().expect("Failed to read output!");
    assert!(!output.status.success());
}

#[test]
fn errors_for_invalid_sarif() {
    let mut sarifup = create_sarifup();
    let stdin = sarifup.stdin.as_mut().unwrap();

    let missing_ver_sarif_str = b"{\"runs\":[{\"tool\":{\"driver\":{\"name\":\"test\"}}}]}";

    stdin.write_all(missing_ver_sarif_str).unwrap();

    let output = sarifup.wait_with_output().expect("Failed to read output!");
    assert!(!output.status.success());
}

#[test]
fn succeeds_for_valid_sarif() {
    let mut sarifup = create_sarifup();
    let stdin = sarifup.stdin.as_mut().unwrap();

    stdin.write_all(VALID_SARIF_STR).unwrap();

    let output = sarifup.wait_with_output().expect("Failed to read output!");
    assert!(output.status.success());
}

#[test]
fn returns_input_for_valid_sarif_input() {
    let mut sarifup = create_sarifup();
    let stdin = sarifup.stdin.as_mut().unwrap();

    stdin.write_all(VALID_SARIF_STR).unwrap();

    let output = sarifup.wait_with_output().expect("Failed to read output!");
    assert_eq!(&output.stdout, VALID_SARIF_STR);
}

#[test]
fn erorrs_when_missing_filename_arg() {
    let mut sarifup = Command::new("target/debug/sarifup")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn app!");

    let stdin = sarifup.stdin.as_mut().unwrap();

    stdin.write_all(VALID_SARIF_STR).unwrap();

    let output = sarifup.wait_with_output().expect("Failed to read output!");
    assert!(!output.status.success());
}
