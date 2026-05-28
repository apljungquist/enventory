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
    for (name, bin) in bins() {
        let stdout = bin().env_clear().run_ok();
        expected.assert_eq(&stdout);
        eprintln!("{name}: ok");
    }
}

#[test]
fn cli_override() {
    let expected = expect![[r#"
        Working with verbose=true, debug=false, trace=false, color=auto, timeout=30s, port=none
        Listening on port 8080
    "#]];
    for (name, bin) in bins() {
        let stdout = bin().env_clear().arg("--example-verbose=true").run_ok();
        expected.assert_eq(&stdout);
        eprintln!("{name}: ok");
    }
}

#[test]
fn env_var_override() {
    let expected = expect![[r#"
        Working with verbose=true, debug=false, trace=false, color=auto, timeout=30s, port=none
        Listening on port 8080
    "#]];
    for (name, bin) in bins() {
        let stdout = bin().env_clear().env("EXAMPLE_VERBOSE", "true").run_ok();
        expected.assert_eq(&stdout);
        eprintln!("{name}: ok");
    }
}

#[test]
fn cli_beats_env() {
    let expected = expect![[r#"
        Working with verbose=true, debug=false, trace=false, color=auto, timeout=30s, port=none
        Listening on port 8080
    "#]];
    for (name, bin) in bins() {
        let stdout = bin()
            .env_clear()
            .env("EXAMPLE_VERBOSE", "false")
            .arg("--example-verbose=true")
            .run_ok();
        expected.assert_eq(&stdout);
        eprintln!("{name}: ok");
    }
}

#[test]
fn boolish_parsing() {
    let expected = expect![[r#"
        Working with verbose=false, debug=true, trace=false, color=auto, timeout=30s, port=none
        Listening on port 8080
    "#]];
    for (name, bin) in bins() {
        let stdout = bin().env_clear().env("EXAMPLE_DEBUG", "on").run_ok();
        expected.assert_eq(&stdout);
        eprintln!("{name}: ok");
    }
}

#[test]
fn parse_error_cli_arg() {
    let expected = expect![[r#"
        error: invalid value 'abc' for '--example-debug <EXAMPLE_DEBUG>': expected a boolean (true/false/yes/no/on/off/1/0), got 'abc'

        For more information, try '--help'.
    "#]];
    for (name, bin) in bins() {
        if name == "reference" {
            continue;
        }
        let stderr = bin().env_clear().arg("--example-debug=abc").run_err();
        expected.assert_eq(&stderr);
        eprintln!("{name}: ok");
    }
    // TODO: align reference's error message with the others.
    let reference_expected = expect![[r#"
        error: invalid value 'abc' for '--example-debug <EXAMPLE_DEBUG>': value was not a boolean

        For more information, try '--help'.
    "#]];
    let stderr = reference().env_clear().arg("--example-debug=abc").run_err();
    reference_expected.assert_eq(&stderr);
}

#[test]
fn parse_error_env_var() {
    let expected = expect![[r#"
        error: invalid value 'abc' for '--example-debug <EXAMPLE_DEBUG>': expected a boolean (true/false/yes/no/on/off/1/0), got 'abc'

        For more information, try '--help'.
    "#]];
    for (name, bin) in bins() {
        if name == "reference" {
            continue;
        }
        let stderr = bin().env_clear().env("EXAMPLE_DEBUG", "abc").run_err();
        expected.assert_eq(&stderr);
        eprintln!("{name}: ok");
    }
    // TODO: align reference's error message with the others.
    let reference_expected = expect![[r#"
        error: invalid value 'abc' for '--example-debug <EXAMPLE_DEBUG>': value was not a boolean

        For more information, try '--help'.
    "#]];
    let stderr = reference()
        .env_clear()
        .env("EXAMPLE_DEBUG", "abc")
        .run_err();
    reference_expected.assert_eq(&stderr);
}

#[test]
fn help_text() {
    // Literal sits at column 0 because clap's help text has multiple indent
    // levels (2/6/10 spaces); expect-test's unindent strips a single common
    // prefix, so anything other than no-common-prefix would corrupt alignment.
    let expected = expect![[r#"
Example binary demonstrating enventory

Usage: example-bin [OPTIONS]

Options:
      --example-color <EXAMPLE_COLOR>
          When to use colored output

          Accepts: auto, always, never [env: EXAMPLE_COLOR=] [default: auto]
      --example-debug <EXAMPLE_DEBUG>
          Enable debug mode [env: EXAMPLE_DEBUG=] [default: false]
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
    for (name, bin) in bins() {
        if name == "reference" {
            continue;
        }
        let stdout = bin().env_clear().arg("--help").run_ok();
        // Clap pads description blank lines with trailing whitespace; trim it
        // so the snapshot can be expressed as a truly empty line.
        let stdout = strip_trailing_ws(&stdout);
        expected.assert_eq(&stdout);
        eprintln!("{name}: ok");
    }
    // TODO: align reference's help text with the others (field order, default
    //  metavars, possible-values annotations).
    let reference_expected = expect![[r#"
Example binary demonstrating enventory

Usage: example-bin [OPTIONS]

Options:
      --example-verbose <EXAMPLE_VERBOSE>
          [env: EXAMPLE_VERBOSE=] [default: false] [possible values: true, false]
      --example-debug <EXAMPLE_DEBUG>
          Enable debug mode [env: EXAMPLE_DEBUG=] [default: false] [possible values: true, false]
      --example-trace <EXAMPLE_TRACE>
          Enable trace mode [env: EXAMPLE_TRACE=] [default: false] [possible values: true, false]
      --example-color <EXAMPLE_COLOR>
          When to use colored output [env: EXAMPLE_COLOR=] [default: auto]
      --example-timeout <EXAMPLE_TIMEOUT>
          Request timeout [env: EXAMPLE_TIMEOUT=] [default: 30s]
      --example-port <EXAMPLE_PORT>
          Optional port override [env: EXAMPLE_PORT=]
      --port <PORT>
          Port to listen on [env: PORT=] [default: 8080]
  -h, --help
          Print help
"#]];
    let stdout = strip_trailing_ws(&reference().env_clear().arg("--help").run_ok());
    reference_expected.assert_eq(&stdout);
}

fn strip_trailing_ws(s: &str) -> String {
    let mut out: String = s.lines().map(str::trim_end).collect::<Vec<_>>().join("\n");
    if s.ends_with('\n') {
        out.push('\n');
    }
    out
}
