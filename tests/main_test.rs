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
    br#"{"runs":[{"tool":{"driver":{"name":"test"}}}],"version":"1.0.0"}"#;

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

    let missing_ver_sarif_str = br#"{"runs":[{"tool":{"driver":{"name":"test"}}}]}"#;
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
fn erorrs_with_message_when_cannot_open_sarif_file() {
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

#[test]
fn updates_sarif_file() {
    let mut sarifup = Command::new(SARIFUP_PATH)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg("tests/old.sarif")
        .spawn()
        .expect("Failed to spawn app!");

    let stdin = sarifup.stdin.as_mut().unwrap();
    let new_sarif = br#"{
        "version": "2.1.0",
        "runs": [{
            "tool": { "driver": { "name": "new_sarif" } },
            "results": [{
                "message": { "text": "new sarif message" },
                "fingerprints": { "hashResult/v1": "abc123" }
            }, {
                "message": { "text": "new sarif other message" },
                "fingerprints": { "hashResult/v1": "no-match-999" }
            }]
        }]
    }"#;
    stdin.write_all(new_sarif).unwrap();

    let output = sarifup.wait_with_output().unwrap();
    assert!(output.status.success());
    let stdout_str = std::str::from_utf8(&output.stdout).unwrap();
    // copied from old.sarif for matching fingerprint abc123
    assert!(stdout_str.contains("old sarif message"));
    assert!(stdout_str.contains("\"rank\":5"));
    assert!(stdout_str.contains("old justification"));
    // non-matching fingerprint should keep new message and not copy old second entry
    assert!(stdout_str.contains("new sarif other message"));
    assert!(!stdout_str.contains("old sarif other message"));
    assert!(!stdout_str.contains("\"rank\":3"));
    //should show count of new, updated and closed results
    assert!(stdout_str.contains("1 new, 1 updated and 1 closed results."));
}
