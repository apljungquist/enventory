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
//! enventory::define!(
//!     /// Port to listen on
//!     pub static MY_PORT: u16 = 8080
//! );
//!
//! fn serve() {
//!     println!("Serving on port {}", *MY_PORT)
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

mod core;
#[cfg(feature = "clap")]
mod feat_clap;
#[cfg(feature = "inventory")]
mod feat_inventory;
mod macros;

#[cfg(feature = "clap")]
pub use feat_clap::{apply_matches, apply_matches_for, args, EnvArgs};
#[doc(hidden)]
#[cfg(feature = "inventory")]
pub use inventory;

pub use self::core::{parse_boolish, parse_from_str, parse_some, Item};
#[cfg(feature = "clap")]
pub(crate) use self::feat_inventory::check_consistency;
#[cfg(feature = "clap")]
pub(crate) use self::feat_inventory::iter;
#[doc(hidden)]
#[cfg(feature = "inventory")]
pub use self::feat_inventory::ItemEntry;
#[cfg(feature = "inventory")]
pub use self::feat_inventory::{validate_all, ParseError, ValidationErrors};
