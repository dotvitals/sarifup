#[test]
fn errors_with_code_1_for_invalid_json() {
    use std::process::Command;
    use std::process::Stdio;
    let mut sarifup = Command::new("target/debug/sarifup")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn app!");

    let stdin = sarifup.stdin.as_mut().unwrap();

    use std::io::Write;
    stdin.write_all(b"d{").unwrap();

    let output = sarifup.wait_with_output().expect("Failed to read output!");
    assert!(!output.status.success());
    assert_eq!(output.status.code().unwrap(), 1);
}

#[test]
fn outputs_with_success_for_valid_json() {
    use std::process::Command;
    use std::process::Stdio;
    let mut sarifup = Command::new("target/debug/sarifup")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn app!");

    let stdin = sarifup.stdin.as_mut().unwrap();

    use std::io::Write;
    stdin.write_all(b"{\"test\": \"test\"}").unwrap();

    let output = sarifup.wait_with_output().expect("Failed to read output!");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "{\"test\":\"test\"}"
    );
}
