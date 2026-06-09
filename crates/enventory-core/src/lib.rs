//! Cross-crate registration of environment variables.
//!
//! An [`Item`] describes a single registered environment variable.
//! Library crates submit items with [`inventory::submit!`] and
//! binary crates discover them with [`iter`].
//!
//! # Example
//!
//! ```
//! // Register an environment variable, typically from a library
//! inventory::submit! {
//!     enventory_core::Item::new("COLOR")
//!         .with_help("When to print ANSI color codes")
//! }
//!
//! // List registered environment variables, typically in a binary
//! for item in enventory_core::iter() {
//!     // Parse environment variable or print help text
//! }
//! ```
//!
//! # Why a separate crate?
//!
//! [`inventory`] identifies a collection by the concrete type passed to [`inventory::collect!`].
//! If two semver-incompatible versions of the type exist in the same binary, they form two
//! separate collections and items registered against one version are invisible to the other.
//!
//! By isolating the collected type in a small, stable crate that aims to *never* make a breaking
//! change, libraries and binaries can depend on different versions of higher-level crates without
//! splitting the collection.

use std::ffi::OsStr;
use std::fmt;

/// An error returned by a [`Validator`].
#[derive(Debug)]
pub struct SetError {
    source: Box<dyn std::error::Error + Send + Sync>,
}

impl SetError {
    /// Creates a new [`SetError`] from a source that is itself an error.
    pub fn new<E>(source: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self {
            source: source.into(),
        }
    }
}

impl fmt::Display for SetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.source, f)
    }
}

impl std::error::Error for SetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&*self.source)
    }
}

/// A function that parses and caches user input.
pub type Validator = fn(&OsStr) -> Result<(), SetError>;

/// Information about one variant of an environment variable to enable user feedback.
#[derive(Debug)]
pub struct PossibleValue {
    name: &'static str,
    help: Option<&'static str>,
}

impl PossibleValue {
    /// Creates a [`PossibleValue`] for the `name` variant of an environment variable.
    pub const fn new(name: &'static str) -> Self {
        Self { name, help: None }
    }

    /// Sets the description of the variant.
    ///
    /// This should be short and only one line.
    pub const fn with_help(mut self, help: &'static str) -> Self {
        self.help = Some(help);
        self
    }

    /// The name set when the possible value was constructed.
    pub fn name(&self) -> &str {
        self.name
    }

    /// The help text set via [`Self::with_help`], if any.
    pub fn help(&self) -> Option<&str> {
        self.help
    }
}

/// A description of an environment variable.
///
/// This may hold information that supports binary crates in:
/// - eagerly validating input,
/// - explaining to users what the effect of setting the environment variable is, and
/// - explaining to users how to configure the environment variable.
pub struct Item {
    key: &'static str,
    help: Option<&'static str>,
    default_value: Option<fn() -> Option<String>>,
    validator: Option<Validator>,
    possible_values: Option<&'static [PossibleValue]>,
}

impl Item {
    /// Creates an [`Item`] for the environment variable named `key`.
    pub const fn new(key: &'static str) -> Self {
        Self {
            key,
            help: None,
            default_value: None,
            validator: None,
            possible_values: None,
        }
    }

    /// Sets the short, one-line, description of the environment variable.
    pub const fn with_help(mut self, v: &'static str) -> Self {
        self.help = Some(v);
        self
    }

    /// Sets the value to be used when no value is provided by the user.
    pub const fn with_default_value(mut self, f: fn() -> Option<String>) -> Self {
        self.default_value = Some(f);
        self
    }

    /// Sets the [`Validator`] that runs on supplied values.
    ///
    /// The closure should both parse the value and store the result.
    pub const fn with_validator(mut self, f: Validator) -> Self {
        self.validator = Some(f);
        self
    }

    /// Sets the possible values that the validator accepts, if they can be enumerated.
    ///
    /// Used primarily for feedback to the user, not for parsing or validating the input.
    pub const fn with_possible_values(mut self, v: &'static [PossibleValue]) -> Self {
        self.possible_values = Some(v);
        self
    }

    /// The name of the environment variable
    pub fn key(&self) -> &str {
        self.key
    }

    /// The help text set via [`Self::with_help`], if any.
    pub fn help(&self) -> Option<&str> {
        self.help
    }

    /// The default value set via [`Self::with_default_value`], if any.
    pub fn default_value(&self) -> Option<String> {
        self.default_value.and_then(|f| f())
    }

    /// Run the registered validator.
    ///
    /// Returns an error if the validator returned an error.
    pub fn validate(&self, v: &OsStr) -> Result<(), SetError> {
        match self.validator {
            Some(f) => f(v),
            None => Ok(()),
        }
    }

    /// The possible values set via [`Self::with_possible_values`], if any.
    pub fn possible_values(&self) -> Option<&[PossibleValue]> {
        self.possible_values
    }
}

inventory::collect!(Item);

/// Iterator over every [`Item`] submitted.
pub fn iter() -> impl Iterator<Item = &'static Item> {
    inventory::iter::<Item>.into_iter()
}
