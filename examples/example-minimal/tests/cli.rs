//! The expected `--help` block below MUST match the ```text``` block in the
//! `# Example` section of `crates/enventory/src/lib.rs`. The crate-level docs
//! show this output as evidence of the macro generating help text, so if it
//! drifts the documentation becomes a lie. Run with `UPDATE_EXPECT=1` to
//! refresh after intentional changes, then mirror the new text into `lib.rs`.

use expect_test::expect;
use test_utils::RunExt;

fn bin() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_example-minimal"))
}

#[test]
fn help_text() {
    // Literal at column 0 so expect-test's unindent (which strips the smallest
    // common prefix) doesn't mangle clap's already-aligned columns.
    let expected = expect![[r#"
Usage: example-minimal [OPTIONS]

Options:
      --my-port <MY_PORT>  Port to listen on [env: MY_PORT=] [default: 8080]
  -h, --help               Print help
"#]];
    let stdout = bin().env_clear().arg("--help").run_ok();
    expected.assert_eq(&stdout);
}
