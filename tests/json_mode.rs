use serde_json::Value;
use std::process::Command;

fn run_vex(args: &[&str]) -> (String, String, i32) {
    let output = Command::new(env!("CARGO_BIN_EXE_vex"))
        .args(args)
        .output()
        .expect("failed to execute vex");

    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
        output.status.code().unwrap_or(-1),
    )
}

#[test]
fn json_mode_emits_valid_json_object_on_stdout() {
    let (stdout, _stderr, code) = run_vex(&[
        "--target",
        "127.0.0.1",
        "--port",
        "9",
        "--workers",
        "1",
        "--requests",
        "1",
        "--duration",
        "1",
        "--json",
    ]);

    assert_eq!(code, 0, "process should succeed");
    let parsed: Value = serde_json::from_str(stdout.trim()).expect("stdout must be valid JSON");
    assert!(parsed.is_object(), "top-level JSON must be an object");
}

#[test]
fn json_mode_reports_stop_policy_and_drain_fields() {
    let (stdout, _stderr, code) = run_vex(&[
        "--target",
        "127.0.0.1",
        "--port",
        "9",
        "--workers",
        "1",
        "--requests",
        "1",
        "--duration",
        "1",
        "--stop-policy",
        "graceful-drain",
        "--drain-grace-ms",
        "5",
        "--json",
    ]);

    assert_eq!(code, 0, "process should succeed");
    let parsed: Value = serde_json::from_str(stdout.trim()).expect("stdout must be valid JSON");

    assert_eq!(
        parsed.get("stop_policy").and_then(Value::as_str),
        Some("graceful-drain")
    );
    assert!(parsed.get("drain").is_some(), "drain object must exist");
    assert!(
        parsed
            .get("drain")
            .and_then(|d| d.get("started"))
            .and_then(Value::as_bool)
            .is_some(),
        "drain.started must be boolean"
    );
    assert!(
        parsed
            .get("drain")
            .and_then(|d| d.get("completed"))
            .and_then(Value::as_bool)
            .is_some(),
        "drain.completed must be boolean"
    );
}
