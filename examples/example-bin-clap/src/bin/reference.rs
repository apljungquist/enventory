use clap::Parser;
use clap::builder::BoolishValueParser;
use example_lib::{Color, Timeout};

#[derive(Parser)]
#[command(
    bin_name = "example-bin",
    about = "Example binary demonstrating enventory"
)]
struct Cli {
    #[arg(long, env, action = clap::ArgAction::Set, default_value_t = false)]
    example_verbose: bool,

    /// Enable debug mode
    #[arg(
        long,
        env,
        action = clap::ArgAction::Set,
        value_parser = BoolishValueParser::new(),
        default_value = "false",
    )]
    example_debug: bool,

    /// Enable trace mode
    #[arg(
        long,
        env,
        action = clap::ArgAction::Set,
        value_parser = BoolishValueParser::new(),
        default_value = "false",
    )]
    example_trace: bool,

    /// When to use colored output
    #[arg(
        long,
        env,
        value_parser = clap::value_parser!(Color),
        default_value_t = Color::Auto,
    )]
    example_color: Color,

    /// Request timeout
    #[arg(
        long,
        env,
        value_parser = clap::value_parser!(Timeout),
        default_value_t = Timeout::new(30),
    )]
    example_timeout: Timeout,

    /// Optional port override
    #[arg(long, env)]
    example_port: Option<u16>,

    /// Port to listen on
    #[arg(long, env, default_value_t = 8080u16)]
    port: u16,
}

fn main() {
    let cli = Cli::parse();

    example_lib::EXAMPLE_VERBOSE
        .set(cli.example_verbose)
        .unwrap();
    example_lib::EXAMPLE_DEBUG.set(cli.example_debug).unwrap();
    example_lib::EXAMPLE_TRACE.set(cli.example_trace).unwrap();
    example_lib::EXAMPLE_COLOR.set(cli.example_color).unwrap();
    example_lib::EXAMPLE_TIMEOUT
        .set(cli.example_timeout)
        .unwrap();
    example_lib::EXAMPLE_PORT.set(cli.example_port).unwrap();
    example_lib_no_macros::PORT.set(cli.port).unwrap();

    example_lib::do_work();
    example_lib_no_macros::do_work();
}
