//! A typed, self-documenting environment variable registry.
//!
//! **For library authors**:
//! interact with strongly typed constants instead of stringly typed environment variables.
//!
//! **For binary authors**:
//! validate environment variables at startup and surface them in `--help` text.
//!
//! # Example
//!
//! ```
//! // Library
//! // =======
//! // `MY_PORT` is always a `u16`.
//! // If the environment variable is not set or cannot be parsed,
//! // then the `8080` is used as fallback.
//!
//! enventory::define! {
//!     /// Port to listen on
//!     pub static MY_PORT: u16 = 8080
//! }
//!
//! fn serve() {
//!     println!("Listening on port {}", *MY_PORT)
//! }
//!
//! // Binary
//! // ======
//! // Augments the `Cli` with environment variables from dependencies.
//! // These will be eagerly parsed along with the normal args and displayed in help messages.
//!
//! use clap::Parser;
//!
//! #[derive(Parser)]
//! struct Cli {
//!     #[command(flatten)]
//!     env: enventory::EnvArgs,
//! }
//!
//! fn main() {
//!     let _cli = Cli::parse();
//!     serve();
//! }
//! ```
//!
//! Running the program with `--help` prints:
//!
//! ```text
//! Usage: example-minimal [OPTIONS]
//!
//! Options:
//!       --my-port <MY_PORT>  Port to listen on [env: MY_PORT=] [default: 8080]
//!   -h, --help               Print help
//! ```
//!
//! The runnable equivalent of this example lives in `examples/example-minimal`,
//! and its integration test asserts that the `--help` block above matches what
//! the binary actually prints.

#[cfg(feature = "clap")]
mod clap_ext;
#[cfg(feature = "inventory")]
mod inventory_core;
#[cfg(feature = "inventory")]
mod inventory_ext;
mod macros;
mod var_core;

#[cfg(feature = "clap")]
pub use clap_ext::{EnvArgs, apply_matches, apply_matches_for, args};
#[doc(hidden)]
#[cfg(feature = "inventory")]
pub use inventory;

#[cfg(feature = "clap")]
pub(crate) use self::inventory_core::iter;
#[doc(hidden)]
#[cfg(feature = "inventory")]
pub use self::inventory_core::{Item, VarRef};
#[cfg(feature = "clap")]
pub(crate) use self::inventory_ext::check_consistency;
#[cfg(feature = "inventory")]
pub use self::inventory_ext::{ValidationErrors, validate_all};
pub use self::var_core::{ParseError, Var, option_repr, parse_boolish, parse_from_str, parse_some};
