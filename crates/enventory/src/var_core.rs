use std::env::VarError;
use std::fmt::{self, Display};
use std::sync::OnceLock;
use std::{env, ops};

/// Helper used by the `define!` macro to adapt `FromStr` into `fn(&str) -> Result<T, String>`.
pub fn parse_from_str<T: std::str::FromStr>(s: &str) -> Result<T, String>
where
    T::Err: fmt::Display,
{
    s.parse().map_err(|e| format!("{e}"))
}

/// Parses a boolean from common string representations (case-insensitive):
/// `true`, `false`, `yes`, `no`, `on`, `off`, `1`, `0`.
pub fn parse_boolish(s: &str) -> Result<bool, String> {
    match s.to_lowercase().as_str() {
        "true" | "yes" | "on" | "1" => Ok(true),
        "false" | "no" | "off" | "0" => Ok(false),
        _ => Err(format!(
            "expected a boolean (true/false/yes/no/on/off/1/0), got '{s}'"
        )),
    }
}

/// Parses a value into `Some(T)` using `FromStr`. Useful for `Option<T>` variables
/// where the default is `None` and any set value is parsed as the inner type.
pub fn parse_some<T: std::str::FromStr>(s: &str) -> Result<Option<T>, String>
where
    T::Err: fmt::Display,
{
    parse_from_str(s).map(Some)
}

/// Formats an `Option<T>` default for display: `Some(v)` → `Some(v.to_string())`, `None` → `None`.
pub fn option_repr<T: fmt::Display>(v: Option<T>) -> Option<String> {
    v.map(|v| v.to_string())
}

/// A single environment variable that could not be parsed.
#[derive(Debug)]
pub struct ParseError {
    name: &'static str,
    value: String,
    message: String,
}

impl ParseError {
    pub fn name(&self) -> &'static str {
        self.name
    }
    pub fn value(&self) -> &str {
        &self.value
    }
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={:?}: {}", self.name, self.value, self.message)
    }
}

impl std::error::Error for ParseError {}

/// A typed environment variable that is resolved once and then cached.
///
/// Created by the [`crate::define!`] macro. Dereferences to `T`, reading from the
/// environment on first access and returning the cached value thereafter.
pub struct Var<T, E> {
    /// `pub(crate)` so the `VarRef` impl in `inventory_core` can read it.
    pub(crate) key: &'static str,
    parse: fn(&str) -> Result<T, E>,
    default: fn() -> T,
    value: OnceLock<T>,
}

impl<T, E> Var<T, E> {
    pub const fn new(
        key: &'static str,
        parse: fn(&str) -> Result<T, E>,
        default: fn() -> T,
    ) -> Self {
        Self {
            key,
            parse,
            default,
            value: OnceLock::new(),
        }
    }

    pub fn set(&self, v: T) -> Result<(), T> {
        self.value.set(v)
    }

    /// Evaluates the default closure. Used by the `__register_entry!` macro
    /// from user-crate expansion sites; that's why this is `pub` even though
    /// the underlying field is private.
    pub fn default_value(&self) -> T {
        (self.default)()
    }
}

impl<T, E: Display> Var<T, E> {
    pub fn set_from_str(&self, s: &str) -> Result<(), ParseError> {
        match (self.parse)(s) {
            Ok(parsed) => {
                let _ = self.value.set(parsed);
                Ok(())
            }
            Err(e) => Err(ParseError {
                name: self.key,
                value: s.to_owned(),
                message: e.to_string(),
            }),
        }
    }

    // TODO: Consider parsing to_string_lossy
    pub fn set_from_env(&self) -> Result<(), ParseError> {
        match env::var(self.key) {
            Ok(s) => self.set_from_str(&s),
            Err(VarError::NotPresent) => {
                let _ = self.value.set((self.default)());
                Ok(())
            }
            Err(VarError::NotUnicode(_)) => Err(ParseError {
                name: self.key,
                value: "<invalid unicode>".to_owned(),
                message: "environment variable contains invalid unicode".to_owned(),
            }),
        }
    }
}

impl<T, E> ops::Deref for Var<T, E> {
    type Target = T;

    /// Returns the cached value if validation has already run, otherwise
    /// lazily reads the environment variable. If the variable is not set or
    /// cannot be parsed, the default provided in [`crate::define!`] is used.
    fn deref(&self) -> &T {
        self.value.get_or_init(|| {
            if let Ok(s) = env::var(self.key) {
                if let Ok(parsed) = (self.parse)(&s) {
                    return parsed;
                }
            }
            (self.default)()
        })
    }
}
