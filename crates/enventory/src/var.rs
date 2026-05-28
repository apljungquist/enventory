use std::ffi::OsStr;
use std::fmt::{self, Display};
use std::sync::OnceLock;
use std::{env, ops};

/// Adapts `<T as FromStr>::from_str` into the
/// `fn(&str) -> Result<T, String>` signature expected by the `parse = …`
/// modifier on [`define!`](crate::define), stringifying the parser's
/// `T::Err` via [`Display`].
///
/// # Examples
///
/// ```
/// use enventory::parse_from_str;
/// assert_eq!(parse_from_str::<u16>("42"), Ok(42u16));
/// assert!(parse_from_str::<u16>("abc").is_err());
/// ```
pub fn parse_from_str<T: std::str::FromStr>(s: &str) -> Result<T, String>
where
    T::Err: fmt::Display,
{
    s.parse().map_err(|e| format!("{e}"))
}

/// Parses a boolean from common string representations (case-insensitive):
/// `true`, `false`, `yes`, `no`, `on`, `off`, `1`, `0`.
///
/// # Examples
///
/// ```
/// use enventory::parse_boolish;
/// assert_eq!(parse_boolish("yes"), Ok(true));
/// assert_eq!(parse_boolish("OFF"), Ok(false));
/// assert!(parse_boolish("maybe").is_err());
/// ```
pub fn parse_boolish(s: &str) -> Result<bool, String> {
    match s.to_lowercase().as_str() {
        "true" | "yes" | "on" | "1" => Ok(true),
        "false" | "no" | "off" | "0" => Ok(false),
        _ => Err("value was not a boolean".to_owned()),
    }
}

/// Parses a value into `Some(T)` using `FromStr`. Useful for `Option<T>`
/// variables where the default is `None` and any set value is parsed as
/// the inner type.
///
/// # Examples
///
/// ```
/// use enventory::parse_some;
/// assert_eq!(parse_some::<u16>("9090"), Ok(Some(9090u16)));
/// assert!(parse_some::<u16>("not-a-port").is_err());
/// ```
pub fn parse_some<T: std::str::FromStr>(s: &str) -> Result<Option<T>, String>
where
    T::Err: fmt::Display,
{
    parse_from_str(s).map(Some)
}

/// Formats an `Option<T>` default for display: `Some(v)` →
/// `Some(v.to_string())`, `None` → `None`. Pair with [`parse_some`] via
/// the `repr = …` modifier on [`define!`](crate::define) for `Option<T>`
/// variables.
///
/// # Examples
///
/// ```
/// use enventory::option_repr;
/// assert_eq!(option_repr(Some(8080u16)), Some("8080".to_string()));
/// assert_eq!(option_repr(None::<u16>), None);
/// ```
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
    /// The env-var name the parse failed for.
    pub fn name(&self) -> &'static str {
        self.name
    }
    /// The raw input value that the parser rejected.
    pub fn value(&self) -> &str {
        &self.value
    }
    /// The parser's error message (e.g. `Display` of the inner parse
    /// failure).
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ParseError {}

#[cfg(feature = "inventory")]
impl From<ParseError> for enventory_core::SetError {
    fn from(e: ParseError) -> Self {
        enventory_core::SetError::new(e)
    }
}

/// A typed environment variable that is resolved once and then cached.
///
/// Created by the [`crate::define!`] macro. Dereferences to `T`, reading from the
/// environment on first access and returning the cached value thereafter.
pub struct Var<T, E> {
    key: &'static str,
    parse: fn(&str) -> Result<T, E>,
    default: fn() -> T,
    value: OnceLock<T>,
}

impl<T, E> Var<T, E> {
    /// Construct a new `Var`.
    ///
    /// - `key` is the env-var name the [`Deref`](ops::Deref) impl reads
    ///   from (typically `stringify!($name)` when used via
    ///   [`define!`](crate::define)).
    /// - `parse` turns the raw env value into `T`. For types with
    ///   `FromStr`, [`parse_from_str`] is a ready-made adapter.
    /// - `default` produces the fallback used when the env var is unset
    ///   or fails to parse.
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

    /// Manually populate the cache. Returns `Err(v)` if the cache is
    /// already initialized (i.e. a prior `set_from_*` call or [`Deref`](ops::Deref)
    /// has already run). Provided as an escape hatch; most callers use the
    /// `set_from_*` family instead.
    pub fn set(&self, v: T) -> Result<(), T> {
        self.value.set(v)
    }

    /// Returns the env-var name this `Var` reads from — the name passed to
    /// [`Var::new`], or `stringify!($name)` when constructed via
    /// [`define!`](crate::define). `const` so the value can be embedded
    /// in a `static` initializer (as `inventory::submit!` does).
    pub const fn key(&self) -> &'static str {
        self.key
    }

    /// Evaluates the typed default — the `|| $default` closure passed to
    /// [`Var::new`]. Useful for rendering the default in `--help` text
    /// (`Some(format!("{}", VAR.default_value()))`) without unlocking the
    /// `OnceLock` cache.
    pub fn default_value(&self) -> T {
        (self.default)()
    }
}

impl<T, E: Display> Var<T, E> {
    /// Reads the env var and dispatches to [`Self::set_from_os_str`]. Absent
    /// vars fall back to the typed default (no parser round-trip).
    pub fn set_from_env(&self) -> Result<(), ParseError> {
        match env::var_os(self.key) {
            Some(v) => self.set_from_os_str(&v),
            None => {
                let _ = self.value.set((self.default)());
                Ok(())
            }
        }
    }

    /// Parses `s` (with `to_str()` first; non-UTF-8 input returns a
    /// `ParseError`) and caches the result. Companion to
    /// [`Self::set_from_str`] for callers that have a `&OsStr` — typically
    /// `enventory_core::Item::with_validator` closures, which receive
    /// values from `clap` as `&OsStr`.
    pub fn set_from_os_str(&self, s: &OsStr) -> Result<(), ParseError> {
        match s.to_str() {
            Some(s) => self.set_from_str(s),
            None => Err(ParseError {
                name: self.key,
                value: s.to_string_lossy().into_owned(),
                message: "value contains invalid unicode".to_owned(),
            }),
        }
    }

    /// Parses `s` with the registered parser and caches the result.
    /// Returns a [`ParseError`] (carrying the key, the raw input, and the
    /// parser's error message) if the input is rejected. A successful
    /// parse silently no-ops if the cache is already initialized.
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
