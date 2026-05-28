use expect_test::expect;

fn bin() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_example-bin-no-clap"))
}

#[test]
fn defaults() {
    let output = bin().env_clear().output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = expect![[r#"
        Working with verbose=false, debug=false, trace=false, color=auto, timeout=30s, port=none
    "#]];
    expected.assert_eq(&stdout);
}

#[test]
fn env_override() {
    let output = bin()
        .env_clear()
        .env("EXAMPLE_VERBOSE", "true")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = expect![[r#"
        Working with verbose=true, debug=false, trace=false, color=auto, timeout=30s, port=none
    "#]];
    expected.assert_eq(&stdout);
}

#[test]
fn parse_error() {
    let output = bin()
        .env_clear()
        .env("EXAMPLE_DEBUG", "abc")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    // Strip the location-dependent first line ("thread 'main' panicked at ...")
    // and the backtrace hint, keeping only the stable error payload.
    let stable: String = stderr
        .lines()
        .filter(|l| !l.starts_with("thread '") && !l.starts_with("note: run with"))
        .collect::<Vec<_>>()
        .join("\n");
    let expected = expect![[
        r#"validation failed: ValidationErrors { errors: [ParseError { name: "EXAMPLE_DEBUG", type_name: "bool", value: "abc", message: "expected a boolean (true/false/yes/no/on/off/1/0), got 'abc'" }] }"#
    ]];
    expected.assert_eq(&stable);
}
