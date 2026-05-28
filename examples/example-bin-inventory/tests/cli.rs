use expect_test::expect;
use test_utils::RunExt;

fn bin() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_example-bin-inventory"))
}

#[test]
fn defaults() {
    let stdout = bin().env_clear().run_ok();
    let expected = expect![[r#"
        Working with verbose=false, debug=false, trace=false, color=auto, timeout=30s, port=none
        Listening on port 8080
    "#]];
    expected.assert_eq(&stdout);
}

#[test]
fn env_override() {
    let stdout = bin().env_clear().env("EXAMPLE_VERBOSE", "true").run_ok();
    let expected = expect![[r#"
        Working with verbose=true, debug=false, trace=false, color=auto, timeout=30s, port=none
        Listening on port 8080
    "#]];
    expected.assert_eq(&stdout);
}

#[test]
fn parse_error() {
    let stderr = bin().env_clear().env("EXAMPLE_DEBUG", "abc").run_err();
    // Strip the location-dependent first line ("thread 'main' panicked at ...")
    // and the backtrace hint, keeping only the stable error payload.
    let stable: String = stderr
        .lines()
        .filter(|l| !l.starts_with("thread '") && !l.starts_with("note: run with"))
        .collect::<Vec<_>>()
        .join("\n");
    let expected = expect![[r#"

        validation failed: ValidationErrors { errors: [SetError { source: ParseError { name: "EXAMPLE_DEBUG", value: "abc", message: "value was not a boolean" } }] }"#]];
    expected.assert_eq(&stable);
}
