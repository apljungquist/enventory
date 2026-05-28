//! Cross-crate registration machinery: the [`Item`] struct plus the
//! `inventory::collect!` invocation.
//!
//! Most users don't touch this crate directly — `enventory::define!`
//! ([`enventory::define`](../enventory/macro.define.html)) handles
//! registration on their behalf, and `enventory` re-exports the few
//! symbols a binary or hand-rolled `inventory::submit!` block needs
//! ([`Item`], [`PossibleValue`], [`SetError`]). The crate exists as a
//! separate unit so library crates can call `define!` without dragging
//! `inventory` into their own dep tree; registration only activates when
//! a downstream binary enables `enventory`'s `inventory` feature.
//!
//! # Vocabulary
//!
//! [`Item`] and [`PossibleValue`] mirror the shape of `clap::Arg` /
//! `clap::builder::PossibleValue`, but use the naming convention shared
//! across `enventory`: builders are `with_<field>(...)`, accessors are
//! bare `<field>()`. The constructor takes `key` (the env-var name). The
//! one departure from clap's *vocabulary* is [`Item::with_validator`]
//! (clap's `value_parser`): our function returns `Result<(), SetError>` —
//! a verdict, not a parsed value — because the typed value lands in the
//! per-variable `Var<T>` cache as a side effect, not in clap's
//! `ArgMatches`.

use std::ffi::OsStr;
use std::fmt;

/// An error returned by an [`Item`]'s validator when the supplied value
/// cannot be parsed.
///
/// Opaque wrapper around the underlying parse failure. Inspect via
/// [`Display`](fmt::Display), [`Debug`], or
/// [`Error::source`](std::error::Error::source). The minimal shape is
/// deliberate — structured context (env-var name, raw value, …) can be
/// added later as fields without disturbing existing callers.
#[derive(Debug)]
pub struct SetError {
    source: Box<dyn std::error::Error + Send + Sync>,
}

impl SetError {
    /// Wrap any boxable error into a `SetError`. Used by
    /// [`Item::with_validator`] closures to surface parse failure.
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
    /// Construct a [`PossibleValue`] for the `name` variant.
    pub const fn new(name: &'static str) -> Self {
        Self { name, help: None }
    }

    /// Sets the short, one-line, description of the variant.
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

/// Information an environment variable to enable:
/// - eager parsing, and
/// - user feedback.
///
/// # Example
///
/// ```ignore
/// inventory::submit! {
///     enventory::Item::new("COLOR")
///         .with_help("When to print ANSI color codes")
/// }
/// ```
pub struct Item {
    key: &'static str,
    help: Option<&'static str>,
    default_value: Option<fn() -> Option<String>>,
    validator: Option<Validator>,
    possible_values: Option<&'static [PossibleValue]>,
}

impl Item {
    /// Construct an [`Item`] for the environment variable named `key`.
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
