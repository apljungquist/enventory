use expect_test::expect;
use test_utils::RunExt;

fn bin() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_example-bin-no-inventory"))
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
fn invalid_value_falls_back_to_default() {
    let stdout = bin()
        .env_clear()
        .env("EXAMPLE_VERBOSE", "not-a-bool")
        .run_ok();
    let expected = expect![[r#"
        Working with verbose=false, debug=false, trace=false, color=auto, timeout=30s, port=none
        Listening on port 8080
    "#]];
    expected.assert_eq(&stdout);
}
