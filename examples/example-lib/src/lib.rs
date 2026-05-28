use std::fmt;

use clap::builder::{ValueParser, ValueParserFactory};

#[derive(Clone, Debug)]
pub enum Color {
    Auto,
    Always,
    Never,
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Color::Auto => write!(f, "auto"),
            Color::Always => write!(f, "always"),
            Color::Never => write!(f, "never"),
        }
    }
}

impl std::str::FromStr for Color {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(Color::Auto),
            "always" => Ok(Color::Always),
            "never" => Ok(Color::Never),
            _ => Err(format!("expected auto, always, or never, got '{s}'")),
        }
    }
}

// A custom type that doesn't implement FromStr.
#[derive(Clone, Debug)]
pub struct Timeout(u64);

impl Timeout {
    pub const fn new(seconds: u64) -> Self {
        Self(seconds)
    }
}

impl fmt::Display for Timeout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}s", self.0)
    }
}

fn parse_timeout(s: &str) -> Result<Timeout, String> {
    let s = s.strip_suffix('s').unwrap_or(s);
    s.parse::<u64>()
        .map(Timeout)
        .map_err(|e| format!("expected timeout in seconds (e.g. '30' or '30s'), got '{s}': {e}"))
}

impl ValueParserFactory for Color {
    type Parser = ValueParser;
    fn value_parser() -> Self::Parser {
        ValueParser::from(<Color as std::str::FromStr>::from_str)
    }
}

impl ValueParserFactory for Timeout {
    type Parser = ValueParser;
    fn value_parser() -> Self::Parser {
        ValueParser::from(parse_timeout)
    }
}

// The docstring is optional:
enventory::define! {
    pub static EXAMPLE_VERBOSE: bool = false
}

// TODO: Specify possible values to make this equivalent to define_boolish!

// The parser can be overridden:
enventory::define! {
    /// Enable trace mode
    pub static EXAMPLE_TRACE: bool = false, parse = enventory::parse_boolish
}

// Boolean environment variables with flexible parsing can be defined with a shorthand macro:
enventory::define_boolish! {
    /// Enable debug mode
    pub static EXAMPLE_DEBUG: bool = false
}

// Custom types can also be parsed using `FromStr`:
enventory::define! {
    /// Whether to use color
    pub static EXAMPLE_COLOR: Color = Color::Auto
}

// Custom types that don't implement `FromStr` can be parsed with a custom parser:
enventory::define! {
    /// Request timeout
    pub static EXAMPLE_TIMEOUT: Timeout = Timeout(30), parse = parse_timeout
}

// Optional values default to `None` and parse the inner type when set:
enventory::define! {
    /// Optional port override
    pub static EXAMPLE_PORT: Option<u16> = None, parse = enventory::parse_some::<u16>, repr = enventory::option_repr
}

pub fn do_work() {
    let verbose = *EXAMPLE_VERBOSE;
    let debug = *EXAMPLE_DEBUG;
    let trace = *EXAMPLE_TRACE;
    let color = &*EXAMPLE_COLOR;
    let timeout = &*EXAMPLE_TIMEOUT;
    let port = &*EXAMPLE_PORT;
    let port_str = match port {
        Some(p) => p.to_string(),
        None => "none".to_string(),
    };
    println!(
        "Working with verbose={verbose}, debug={debug}, trace={trace}, color={color}, timeout={timeout}, port={port_str}"
    );
}
