use expect_test::expect;

fn bin() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_example-bin-lazy"))
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
fn invalid_value_falls_back_to_default() {
    let output = bin()
        .env_clear()
        .env("EXAMPLE_VERBOSE", "not-a-bool")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = expect![[r#"
        Working with verbose=false, debug=false, trace=false, color=auto, timeout=30s, port=none
    "#]];
    expected.assert_eq(&stdout);
}
