use std::process::Child;

fn create_sarifup() -> Child {
    use std::process::Command;
    use std::process::Stdio;
    Command::new("target/debug/sarifup")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn app!")
}

#[test]
fn errors_with_invalid_json() {
    let mut sarifup = create_sarifup();
    let stdin = sarifup.stdin.as_mut().unwrap();

    use std::io::Write;
    stdin.write_all(b"d{").unwrap();

    let output = sarifup.wait_with_output().expect("Failed to read output!");
    assert!(!output.status.success());
}

#[test]
fn errors_for_invalid_sarif() {
    let mut sarifup = create_sarifup();
    let stdin = sarifup.stdin.as_mut().unwrap();

    let missing_ver_sarif_str = b"{\"runs\":[{\"tool\":{\"driver\":{\"name\":\"test\"}}}]}";

    use std::io::Write;
    stdin.write_all(missing_ver_sarif_str).unwrap();

    let output = sarifup.wait_with_output().expect("Failed to read output!");
    assert!(!output.status.success());
}

#[test]
fn succeeds_for_valid_sarif() {
    let mut sarifup = create_sarifup();
    let stdin = sarifup.stdin.as_mut().unwrap();

    let sarif_str =
        b"{\"runs\":[{\"tool\":{\"driver\":{\"name\":\"test\"}}}],\"version\":\"1.0.0\"}";

    use std::io::Write;
    stdin.write_all(sarif_str).unwrap();

    let output = sarifup.wait_with_output().expect("Failed to read output!");
    assert!(output.status.success());
}

#[test]
fn returns_input_for_valid_sarif() {
    let mut sarifup = create_sarifup();
    let stdin = sarifup.stdin.as_mut().unwrap();

    let sarif_str =
        b"{\"runs\":[{\"tool\":{\"driver\":{\"name\":\"test\"}}}],\"version\":\"1.0.0\"}";

    use std::io::Write;
    stdin.write_all(sarif_str).unwrap();

    let output = sarifup.wait_with_output().expect("Failed to read output!");
    assert_eq!(&output.stdout, sarif_str);
}
