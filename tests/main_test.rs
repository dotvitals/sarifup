#[test]
fn main_pipes_stdin_to_stdout() {
    use std::process::Command;
    use std::process::Stdio;
    let mut sarifup = Command::new("target/debug/sarifup")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn app!");

    let stdin = sarifup.stdin.as_mut().unwrap();

    use std::io::Write;
    stdin.write_all(b"42\n").unwrap();

    let output = sarifup.wait_with_output().expect("Failed to read output!");
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "42\n");
}
