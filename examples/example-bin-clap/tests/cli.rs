//! Apply pressure on `enventory` to make the result of using the clap-integration look and feel
//! like the CLI had been defined directly with clap.
//!
//! Note that it is not a goal that every `clap` API can be defined with `enventory`.
//!
//! This particular test ensures that one reasonable CLI can be expressed with `enventory` and that
//! it behaves the same as the `clap` reference.
use expect_test::expect;
use test_utils::RunExt;

fn declarative() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_declarative"))
}

fn procedural() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_procedural"))
}

fn reference() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_reference"))
}

type BinEntry = (&'static str, fn() -> std::process::Command);

fn bins() -> Vec<BinEntry> {
    vec![
        ("declarative", declarative),
        ("procedural", procedural),
        ("reference", reference),
    ]
}

#[test]
fn defaults() {
    let expected = expect![[r#"
        Working with verbose=false, debug=false, trace=false, color=auto, timeout=30s, port=none
        Listening on port 8080
    "#]];
    let mut first: Option<String> = None;
    for (i, (name, bin)) in bins().into_iter().enumerate() {
        let stdout = bin().env_clear().run_ok();
        if i == 0 {
            expected.assert_eq(&stdout);
            first = Some(stdout);
        } else {
            assert_eq!(first.as_deref(), Some(stdout.as_str()));
        }
        eprintln!("{name}: ok");
    }
}

#[test]
fn cli_override() {
    let expected = expect![[r#"
        Working with verbose=true, debug=false, trace=false, color=auto, timeout=30s, port=none
        Listening on port 8080
    "#]];
    let mut first: Option<String> = None;
    for (i, (name, bin)) in bins().into_iter().enumerate() {
        let stdout = bin().env_clear().arg("--example-verbose=true").run_ok();
        if i == 0 {
            expected.assert_eq(&stdout);
            first = Some(stdout);
        } else {
            assert_eq!(first.as_deref(), Some(stdout.as_str()));
        }
        eprintln!("{name}: ok");
    }
}

#[test]
fn env_var_override() {
    let expected = expect![[r#"
        Working with verbose=true, debug=false, trace=false, color=auto, timeout=30s, port=none
        Listening on port 8080
    "#]];
    let mut first: Option<String> = None;
    for (i, (name, bin)) in bins().into_iter().enumerate() {
        let stdout = bin().env_clear().env("EXAMPLE_VERBOSE", "true").run_ok();
        if i == 0 {
            expected.assert_eq(&stdout);
            first = Some(stdout);
        } else {
            assert_eq!(first.as_deref(), Some(stdout.as_str()));
        }
        eprintln!("{name}: ok");
    }
}

#[test]
fn cli_beats_env() {
    let expected = expect![[r#"
        Working with verbose=true, debug=false, trace=false, color=auto, timeout=30s, port=none
        Listening on port 8080
    "#]];
    let mut first: Option<String> = None;
    for (i, (name, bin)) in bins().into_iter().enumerate() {
        let stdout = bin()
            .env_clear()
            .env("EXAMPLE_VERBOSE", "false")
            .arg("--example-verbose=true")
            .run_ok();
        if i == 0 {
            expected.assert_eq(&stdout);
            first = Some(stdout);
        } else {
            assert_eq!(first.as_deref(), Some(stdout.as_str()));
        }
        eprintln!("{name}: ok");
    }
}

#[test]
fn boolish_parsing() {
    let expected = expect![[r#"
        Working with verbose=false, debug=true, trace=false, color=auto, timeout=30s, port=none
        Listening on port 8080
    "#]];
    let mut first: Option<String> = None;
    for (i, (name, bin)) in bins().into_iter().enumerate() {
        let stdout = bin().env_clear().env("EXAMPLE_DEBUG", "on").run_ok();
        if i == 0 {
            expected.assert_eq(&stdout);
            first = Some(stdout);
        } else {
            assert_eq!(first.as_deref(), Some(stdout.as_str()));
        }
        eprintln!("{name}: ok");
    }
}

#[test]
fn parse_error_cli_arg() {
    let expected = expect![[r#"
        error: invalid value 'abc' for '--example-debug <EXAMPLE_DEBUG>'
          [possible values: true, false, yes, no, on, off, 1, 0]

        For more information, try '--help'.
    "#]];
    let mut first: Option<String> = None;
    for (i, (name, bin)) in bins().into_iter().enumerate() {
        let stderr = bin().env_clear().arg("--example-debug=abc").run_err();
        if i == 0 {
            expected.assert_eq(&stderr);
            first = Some(stderr);
        } else {
            assert_eq!(first.as_deref(), Some(stderr.as_str()));
        }
        eprintln!("{name}: ok");
    }
}

#[test]
fn parse_error_env_var() {
    let expected = expect![[r#"
        error: invalid value 'abc' for '--example-debug <EXAMPLE_DEBUG>'
          [possible values: true, false, yes, no, on, off, 1, 0]

        For more information, try '--help'.
    "#]];
    let mut first: Option<String> = None;
    for (i, (name, bin)) in bins().into_iter().enumerate() {
        let stderr = bin().env_clear().env("EXAMPLE_DEBUG", "abc").run_err();
        if i == 0 {
            expected.assert_eq(&stderr);
            first = Some(stderr);
        } else {
            assert_eq!(first.as_deref(), Some(stderr.as_str()));
        }
        eprintln!("{name}: ok");
    }
}

#[test]
fn help_text() {
    let expected = expect![[r#"
        Example binary demonstrating enventory

        Usage: example-bin [OPTIONS]

        Options:
              --example-color <EXAMPLE_COLOR>
                  Whether to use color [env: EXAMPLE_COLOR=] [default: auto]
              --example-debug <EXAMPLE_DEBUG>
                  Enable debug mode [env: EXAMPLE_DEBUG=] [default: false] [possible values: true, false, yes, no, on, off, 1, 0]
              --example-port <EXAMPLE_PORT>
                  Optional port override [env: EXAMPLE_PORT=]
              --example-timeout <EXAMPLE_TIMEOUT>
                  Request timeout [env: EXAMPLE_TIMEOUT=] [default: 30s]
              --example-trace <EXAMPLE_TRACE>
                  Enable trace mode [env: EXAMPLE_TRACE=] [default: false]
              --example-verbose <EXAMPLE_VERBOSE>
                  [env: EXAMPLE_VERBOSE=] [default: false]
              --port <PORT>
                  Port to listen on [env: PORT=] [default: 8080]
          -h, --help
                  Print help
    "#]];
    let mut first: Option<String> = None;
    for (i, (name, bin)) in bins().into_iter().enumerate() {
        let stdout = bin().env_clear().arg("--help").run_ok();
        // Clap pads description blank lines with trailing whitespace; trim it
        // so the snapshot can be expressed as a truly empty line.
        let stdout = strip_trailing_ws(&stdout);
        if i == 0 {
            expected.assert_eq(&stdout);
            first = Some(stdout);
        } else {
            assert_eq!(first.as_deref(), Some(stdout.as_str()));
        }
        eprintln!("{name}: ok");
    }
}

fn strip_trailing_ws(s: &str) -> String {
    let mut out: String = s.lines().map(str::trim_end).collect::<Vec<_>>().join("\n");
    if s.ends_with('\n') {
        out.push('\n');
    }
    out
}
