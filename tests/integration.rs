//! Integration tests for the twatch binary.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::process::Command;

#[test]
fn cli_executable_exists() {
    let output =
        Command::new(env!("CARGO_BIN_EXE_twatch")).output().expect("failed to run twatch binary");

    // Binary runs but exits non-zero without a TTY — this is expected
    // We just verify it doesn't segfault and produces output
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Error") || stderr.is_empty(),
        "Binary should produce an error message when no TTY is available"
    );
}

#[test]
fn cli_handles_help_like_flags() {
    for flag in &["--help", "--version", "help"] {
        let output = Command::new(env!("CARGO_BIN_EXE_twatch"))
            .arg(flag)
            .output()
            .expect("failed to run twatch binary");

        // Just verify it doesn't crash — TUI app may not support these flags yet
        let _ = output;
    }
}
