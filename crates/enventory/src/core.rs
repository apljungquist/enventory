use std::sync::OnceLock;
use std::{env, fmt, ops};

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

/// A typed environment variable that is resolved once and then cached.
///
/// Created by the [`crate::define!`] macro. Dereferences to `T`, reading from the
/// environment on first access and returning the cached value thereafter.
pub struct Item<T> {
    pub(crate) name: &'static str,
    #[cfg(feature = "inventory")]
    pub(crate) type_name: &'static str,
    pub(crate) default: fn() -> T,
    pub(crate) parse: fn(&str) -> Result<T, String>,
    pub(crate) value: OnceLock<T>,
}

impl<T> Item<T> {
    #[doc(hidden)]
    pub const fn new(
        name: &'static str,
        #[cfg(feature = "inventory")] type_name: &'static str,
        #[cfg(not(feature = "inventory"))] _type_name: &'static str,
        default: fn() -> T,
        parse: fn(&str) -> Result<T, String>,
    ) -> Self {
        Self {
            name,
            #[cfg(feature = "inventory")]
            type_name,
            default,
            parse,
            value: OnceLock::new(),
        }
    }

    /// Returns the environment variable name (e.g. `"MYHTTP_PORT"`).
    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl<T> ops::Deref for Item<T> {
    type Target = T;

    /// Returns the cached value if validation has already run, otherwise
    /// lazily reads the environment variable. If the variable is not set or
    /// cannot be parsed, the default provided in [`crate::define!`] is used.
    fn deref(&self) -> &T {
        self.value.get_or_init(|| {
            if let Ok(val) = env::var(self.name) {
                if let Ok(parsed) = (self.parse)(&val) {
                    return parsed;
                }
            }
            (self.default)()
        })
    }
}
