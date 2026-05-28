//! Apply pressure on `enventory` to make it easy for clap users to use clap on both the library and
//! the binary side without loss of functionality.

// TODO: Provide the glue that allows ValueEnum to be used as part of `enventory`

use std::str::FromStr;

use clap::{Command, ValueEnum};
use enventory::{Item, PossibleValue, Var, inventory};
use expect_test::expect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Color {
    /// Auto-detect based on the terminal.
    Auto,
    /// Always emit color codes.
    Always,
    /// Never emit color codes.
    Never,
}

impl FromStr for Color {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <Self as ValueEnum>::from_str(s, false)
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        ValueEnum::to_possible_value(self)
            .unwrap()
            .get_name()
            .fmt(f)
    }
}

static COLOR: Var<Color, String> = Var::new("COLOR", <Color as FromStr>::from_str, || Color::Auto);

inventory::submit! {
    Item::new(COLOR.key())
        .with_help("When to use color output")
        .with_default_value(|| Some(format!("{}", COLOR.default_value())))
        .with_validator(|s| COLOR.set_from_os_str(s).map_err(Into::into))
        // clap-derive strips trailing periods from short help (see
        // `clap_derive::utils::doc_comments::remove_period`), so we do the
        // same here to keep the rendered output aligned with the direct
        // side built via `value_parser!(Color)`.
        .with_possible_values(&[
            PossibleValue::new("auto").with_help("Auto-detect based on the terminal"),
            PossibleValue::new("always").with_help("Always emit color codes"),
            PossibleValue::new("never").with_help("Never emit color codes"),
        ])
}

fn enventory_color_arg() -> clap::Arg {
    enventory::args()
        .into_iter()
        .find(|a| a.get_id() == "COLOR")
        .unwrap()
}

fn direct_color_arg() -> clap::Arg {
    clap::Arg::new("COLOR")
        .long("color")
        .env("COLOR")
        .help("When to use color output")
        .default_value("auto")
        .value_parser(clap::value_parser!(Color))
}

fn enventory_color_cmd() -> Command {
    Command::new("test").arg(enventory_color_arg())
}

fn direct_color_cmd() -> Command {
    Command::new("test").arg(direct_color_arg())
}

fn subjects() -> [Command; 2] {
    [enventory_color_cmd(), direct_color_cmd()]
}

const VALID_INPUT: &str = "always";
const INVALID_INPUT: &str = "purple";

#[test]
fn help_long_renders_the_same() {
    let expected = expect![[r#"
        Usage: test [OPTIONS]

        Options:
              --color <COLOR>
                  When to use color output

                  Possible values:
                  - auto:   Auto-detect based on the terminal
                  - always: Always emit color codes
                  - never:  Never emit color codes
                  
                  [env: COLOR=]
                  [default: auto]

          -h, --help
                  Print help (see a summary with '-h')
    "#]];
    for cmd in subjects() {
        let output = cmd
            .try_get_matches_from(["test", "--help"])
            .unwrap_err()
            .to_string();
        expected.assert_eq(&output);
    }
}

#[test]
fn help_short_renders_the_same() {
    let expected = expect![[r#"
        Usage: test [OPTIONS]

        Options:
              --color <COLOR>  When to use color output [env: COLOR=] [default: auto] [possible values: auto, always, never]
          -h, --help           Print help (see more with '--help')
    "#]];
    for cmd in subjects() {
        let output = cmd
            .try_get_matches_from(["test", "-h"])
            .unwrap_err()
            .to_string();
        expected.assert_eq(&output);
    }
}

#[test]
fn invalid_input_yields_same_error_kind() {
    let expected = expect!["InvalidValue"];
    for cmd in subjects() {
        let err = cmd
            .try_get_matches_from(["test", "--color", INVALID_INPUT])
            .unwrap_err();
        expected.assert_eq(&format!("{:?}", err.kind()));
    }
}

#[test]
fn invalid_input_yields_same_error_message() {
    let expected = expect![[r#"
        error: invalid value 'purple' for '--color <COLOR>'
          [possible values: auto, always, never]

        For more information, try '--help'.
    "#]];
    for cmd in subjects() {
        let err = cmd
            .try_get_matches_from(["test", "--color", INVALID_INPUT])
            .unwrap_err();
        expected.assert_eq(&err.to_string());
    }
}

// Once validation succeeds the `OnceLock` is set and subsequent validations will not affect it.
// Consequently we can have at most one happy path test per variable.
// TODO: Find a better way of testing successful validation

#[test]
fn valid_input_parses_to_same_value() {
    let valid_argv = ["test", "--color", VALID_INPUT];

    enventory_color_cmd()
        .try_get_matches_from(valid_argv)
        .unwrap();
    assert_eq!(*COLOR, Color::Always);

    let direct_matches = direct_color_cmd().try_get_matches_from(valid_argv).unwrap();
    assert_eq!(
        direct_matches.get_one::<Color>("COLOR"),
        Some(&Color::Always),
    );
}
