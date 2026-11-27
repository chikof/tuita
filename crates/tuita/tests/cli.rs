//! End-to-end tests that run the compiled binary.

use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tuita"))
        .args(args)
        .env_remove("DATABASE_URL")
        .output()
        .expect("binary starts")
}

#[test]
fn help_lists_every_subcommand() {
    let out = run(&["--help"]);
    assert!(out.status.success());
    let help = String::from_utf8_lossy(&out.stdout);
    for command in ["serve", "migrate", "admin", "seed"] {
        assert!(
            help.lines()
                .any(|line| line.trim_start().starts_with(command)),
            "{command} missing from:\n{help}"
        );
    }
}

#[test]
fn admin_help_lists_actions() {
    let out = run(&["admin", "--help"]);
    assert!(out.status.success());
    let help = String::from_utf8_lossy(&out.stdout);
    for action in [
        "promote",
        "moderator",
        "demote",
        "verify",
        "unverify",
        "suspend",
        "unsuspend",
        "list",
    ] {
        assert!(help.contains(action), "{action} missing from:\n{help}");
    }
}

#[test]
fn admin_without_database_url_fails_with_message() {
    let out = run(&["admin", "promote", "ada"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("invalid DATABASE_URL"), "{stderr}");
}

#[test]
fn unknown_subcommand_is_a_usage_error() {
    let out = run(&["frobnicate"]);
    assert_eq!(out.status.code(), Some(2));
}
