use clap::Parser;
use clap::builder::{PossibleValuesParser, TypedValueParser, ValueParser};
use example_lib::{Color, Timeout};

fn boolish_value_parser_with_possible_values() -> ValueParser {
    PossibleValuesParser::new(["true", "false", "yes", "no", "on", "off", "1", "0"])
        .map(|s: String| matches!(s.as_str(), "true" | "yes" | "on" | "1"))
        .into()
}

fn boolish_value_parser_without_possible_values() -> ValueParser {
    fn parse(s: &str) -> Result<bool, String> {
        match s.to_lowercase().as_str() {
            "true" | "yes" | "on" | "1" => Ok(true),
            "false" | "no" | "off" | "0" => Ok(false),
            _ => Err("value was not a boolean".to_owned()),
        }
    }
    ValueParser::from(parse as fn(&str) -> Result<bool, String>)
}

fn bool_value_parser_without_possible_values() -> ValueParser {
    ValueParser::from(<bool as std::str::FromStr>::from_str)
}

#[derive(Parser)]
#[command(
    bin_name = "example-bin",
    about = "Example binary demonstrating enventory"
)]
struct Cli {
    /// Whether to use color
    #[arg(
        long,
        env,
        value_parser = clap::value_parser!(Color),
        default_value_t = Color::Auto,
    )]
    example_color: Color,

    /// Enable debug mode
    #[arg(
        long,
        env,
        action = clap::ArgAction::Set,
        value_parser = boolish_value_parser_with_possible_values(),
        default_value_t = false,
    )]
    example_debug: bool,

    /// Optional port override
    #[arg(long, env)]
    example_port: Option<u16>,

    /// Request timeout
    #[arg(
        long,
        env,
        value_parser = clap::value_parser!(Timeout),
        default_value_t = Timeout::new(30),
    )]
    example_timeout: Timeout,

    /// Enable trace mode
    #[arg(
        long,
        env,
        action = clap::ArgAction::Set,
        value_parser = boolish_value_parser_without_possible_values(),
        default_value_t = false,
    )]
    example_trace: bool,

    #[arg(
        long,
        env,
        action = clap::ArgAction::Set,
        value_parser = bool_value_parser_without_possible_values(),
        default_value_t = false,
    )]
    example_verbose: bool,

    /// Port to listen on
    #[arg(long, env, default_value_t = 8080u16)]
    port: u16,
}

fn main() {
    let cli = Cli::parse();

    example_lib::EXAMPLE_COLOR.set(cli.example_color).unwrap();
    example_lib::EXAMPLE_DEBUG.set(cli.example_debug).unwrap();
    example_lib::EXAMPLE_PORT.set(cli.example_port).unwrap();
    example_lib::EXAMPLE_TIMEOUT
        .set(cli.example_timeout)
        .unwrap();
    example_lib::EXAMPLE_TRACE.set(cli.example_trace).unwrap();
    example_lib::EXAMPLE_VERBOSE
        .set(cli.example_verbose)
        .unwrap();
    example_lib_no_macros::PORT.set(cli.port).unwrap();

    example_lib::do_work();
    example_lib_no_macros::do_work();
}
