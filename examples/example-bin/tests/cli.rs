use expect_test::expect;

fn declarative() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_example-bin-declarative"))
}

fn procedural() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_example-bin-procedural"))
}

type BinEntry = (&'static str, fn() -> std::process::Command);

fn bins() -> Vec<BinEntry> {
    vec![("declarative", declarative), ("procedural", procedural)]
}

#[test]
fn defaults() {
    for (name, bin) in bins() {
        let output = bin().env_clear().output().unwrap();
        assert!(output.status.success(), "{name} failed");
        let stdout = String::from_utf8(output.stdout).unwrap();
        let expected = expect![[r#"
            Working with verbose=false, debug=false, trace=false, color=auto, timeout=30s, port=none
        "#]];
        expected.assert_eq(&stdout);
    }
}

#[test]
fn cli_override() {
    for (name, bin) in bins() {
        let output = bin()
            .env_clear()
            .arg("--example-verbose=true")
            .output()
            .unwrap();
        assert!(output.status.success(), "{name} failed");
        let stdout = String::from_utf8(output.stdout).unwrap();
        let expected = expect![[r#"
            Working with verbose=true, debug=false, trace=false, color=auto, timeout=30s, port=none
        "#]];
        expected.assert_eq(&stdout);
    }
}

#[test]
fn env_var_override() {
    for (name, bin) in bins() {
        let output = bin()
            .env_clear()
            .env("EXAMPLE_VERBOSE", "true")
            .output()
            .unwrap();
        assert!(output.status.success(), "{name} failed");
        let stdout = String::from_utf8(output.stdout).unwrap();
        let expected = expect![[r#"
            Working with verbose=true, debug=false, trace=false, color=auto, timeout=30s, port=none
        "#]];
        expected.assert_eq(&stdout);
    }
}

#[test]
fn cli_beats_env() {
    for (name, bin) in bins() {
        let output = bin()
            .env_clear()
            .env("EXAMPLE_VERBOSE", "false")
            .arg("--example-verbose=true")
            .output()
            .unwrap();
        assert!(output.status.success(), "{name} failed");
        let stdout = String::from_utf8(output.stdout).unwrap();
        let expected = expect![[r#"
            Working with verbose=true, debug=false, trace=false, color=auto, timeout=30s, port=none
        "#]];
        expected.assert_eq(&stdout);
    }
}

#[test]
fn boolish_parsing() {
    for (name, bin) in bins() {
        let output = bin()
            .env_clear()
            .env("EXAMPLE_DEBUG", "on")
            .output()
            .unwrap();
        assert!(output.status.success(), "{name} failed");
        let stdout = String::from_utf8(output.stdout).unwrap();
        let expected = expect![[r#"
            Working with verbose=false, debug=true, trace=false, color=auto, timeout=30s, port=none
        "#]];
        expected.assert_eq(&stdout);
    }
}

#[test]
fn parse_error() {
    for (name, bin) in bins() {
        let output = bin()
            .env_clear()
            .arg("--example-debug=abc")
            .output()
            .unwrap();
        assert!(!output.status.success(), "{name} should fail");
        let stderr = String::from_utf8(output.stderr).unwrap();
        let expected = expect![[r#"
            error: EXAMPLE_DEBUG="abc": failed to parse as bool: expected a boolean (true/false/yes/no/on/off/1/0), got 'abc'


            Usage: example-bin [OPTIONS]

            For more information, try '--help'.
        "#]];
        expected.assert_eq(&stderr);
    }
}

#[test]
fn help_text() {
    for (name, bin) in bins() {
        let output = bin().env_clear().arg("--help").output().unwrap();
        assert!(output.status.success(), "{name} failed");
        let stdout = String::from_utf8(output.stdout).unwrap();
        let expected = expect![[r#"
            Example binary demonstrating enventory

            Usage: example-bin [OPTIONS]

            Options:
                  --example-color <EXAMPLE_COLOR>
                      When to use colored output [env: EXAMPLE_COLOR=] [default: Color::Auto]
                  --example-debug <EXAMPLE_DEBUG>
                      Enable debug mode [env: EXAMPLE_DEBUG=] [default: false]
                  --example-port <EXAMPLE_PORT>
                      Optional port override [env: EXAMPLE_PORT=] [default: None]
                  --example-timeout <EXAMPLE_TIMEOUT>
                      Request timeout [env: EXAMPLE_TIMEOUT=] [default: Timeout(30)]
                  --example-trace <EXAMPLE_TRACE>
                      Enable trace mode [env: EXAMPLE_TRACE=] [default: false]
                  --example-verbose <EXAMPLE_VERBOSE>
                      [env: EXAMPLE_VERBOSE=] [default: false]
              -h, --help
                      Print help
        "#]];
        expected.assert_eq(&stdout);
    }
}
