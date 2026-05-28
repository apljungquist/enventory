//! A typed, self-documenting environment-variable registry.
//!
//! # For library authors
//!
//! Declare typed env vars with [`define!`] that are parsed once:
//!
//! ```
//! enventory::define! {
//!     /// Port to listen on
//!     pub static MY_PORT: u16 = 8080
//! }
//!
//! fn serve() {
//!     println!("listening on {}", *MY_PORT);
//! }
//! ```
//!
//! # For binary authors
//!
//! - Validate environments eagerly to surface configuration errors that would otherwise cause
//!   surprising behavior, such as "0" being treated as true.
//! - Show users what environment variables may affect the program execution right in the program's
//!   help text.
//!
//! ```
//! use clap::Parser;
//!
//! # enventory::define! {
//! #     /// Port to listen on
//! #     pub static MY_PORT: u16 = 8080
//! # }
//! #
//! # fn serve() {
//! #     println!("listening on {}", *MY_PORT);
//! # }
//!
//! #[derive(Parser)]
//! struct Cli {
//!     #[command(flatten)]
//!     env: enventory::EnvArgs,
//! }
//!
//! fn main() {
//!     let _ = Cli::try_parse_from(["app"]).unwrap();
//!     serve(); // Prints "listening on 8080" if MY_PORT is not set
//! }
//!
//! ```
//!
//! This program would immediately if MY_PORT is not a valid u16 and print the following when
//! invoked with `--help`:
//!
//! ```text
//! Options:
//!       --my-port <MY_PORT>  Port to listen on [env: MY_PORT=] [default: 8080]
//!   -h, --help               Print help
//! ```

#[cfg(feature = "clap")]
mod clap_ext;
#[cfg(feature = "inventory")]
mod env_ext;
mod macros;
mod var;

#[cfg(feature = "clap")]
pub use clap_ext::{EnvArgs, args, set_all_from_matches};
#[doc(hidden)]
#[cfg(feature = "inventory")]
pub use enventory_core::Item;
#[cfg(feature = "clap")]
pub(crate) use enventory_core::iter;
#[cfg(feature = "inventory")]
pub use enventory_core::{PossibleValue, SetError};
#[doc(hidden)]
#[cfg(feature = "inventory")]
pub use inventory;

#[cfg(feature = "clap")]
pub(crate) use self::env_ext::check_consistency;
#[cfg(feature = "inventory")]
pub use self::env_ext::{ValidationErrors, set_all_from_env};
pub use self::var::{ParseError, Var, option_repr, parse_boolish, parse_from_str, parse_some};
