#![cfg(target_os = "macos")]

use std::io::Write;
use std::process::{Command, Output, Stdio};

fn run(args: &[&str], input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_zcco"))
        .args(args)
        .env("PATH", "/nonexistent-zcco-test-path")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = child.stdin.take().unwrap().write_all(input);
    child.wait_with_output().unwrap()
}

#[test]
fn help_and_version_work_without_external_commands() {
    for arg in ["-h", "--help", "-V", "--version"] {
        let output = run(&[arg], b"");
        assert!(output.status.success(), "{:?}", output);
        assert!(String::from_utf8_lossy(&output.stdout).contains("zcco"));
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn unknown_arguments_fail_without_printing_success() {
    let output = run(&["--unknown"], b"hello");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

#[test]
fn full_and_quiet_flags_are_accepted_in_either_spelling_and_order() {
    // Invalid UTF-8 rejects input before clipboard access, letting this test
    // exercise argument parsing without modifying the general pasteboard.
    for args in [
        vec!["-f"],
        vec!["--full"],
        vec!["-q"],
        vec!["--quiet"],
        vec!["-q", "-f"],
        vec!["--full", "--quiet"],
    ] {
        let output = run(&args, &[0xff]);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("读取 UTF-8 标准输入失败"));
    }
}
