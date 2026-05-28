//! Apply pressure on enventory to graciously handle the same environment variable being defined
//! multiple times, possibly with conflicting metadata.

// TODO: Make `enventory` return an error instead of panicking

// TODO: Allow the binary to resolve conflicts

use std::num::ParseIntError;
use std::str::FromStr;

use enventory::{Item, Var, inventory};

static PORT: Var<u16, ParseIntError> = Var::new(
    "ENVENTORY_TEST_DUPLICATE_DIFFERENT_PORT",
    u16::from_str,
    || 8080,
);

inventory::submit! {
    Item::new(PORT.key())
        .with_help("Port to listen on")
        .with_default_value(|| Some(PORT.default_value().to_string()))
        .with_validator(|s| PORT.set_from_os_str(s).map_err(Into::into))
}

inventory::submit! {
    Item::new(PORT.key())
        .with_help("HTTP listen port, defaults to 8080")
        .with_default_value(|| Some(PORT.default_value().to_string()))
        .with_validator(|s| PORT.set_from_os_str(s).map_err(Into::into))
}

#[test]
#[should_panic(
    expected = "conflicting registrations for environment variable 'ENVENTORY_TEST_DUPLICATE_DIFFERENT_PORT'"
)]
fn set_all_from_env_rejects_conflicting_registrations_by_default() {
    let _ = enventory::set_all_from_env();
}

#[test]
#[should_panic(
    expected = "conflicting registrations for environment variable 'ENVENTORY_TEST_DUPLICATE_DIFFERENT_PORT'"
)]
fn try_parse_from_env_args_rejects_conflicting_registrations_by_default() {
    use clap::Parser;

    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        env: enventory::EnvArgs,
    }

    let _ = Cli::try_parse_from(["test"]);
}
