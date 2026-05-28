/// Declares an environment-variable–backed typed static and (with the
/// `inventory` feature) registers it for cross-crate discovery by
/// [`set_all_from_env`](crate::set_all_from_env) and
/// [`EnvArgs`](crate::EnvArgs).
///
/// The env-var name is `stringify!($name)` — declare `MY_PORT` and the
/// process reads the env var `MY_PORT`. The static derefs to `T`:
/// `*MY_PORT` lazily reads the env var on first access, parses it, and
/// caches the result for subsequent reads. On parse failure or missing
/// env, the typed default is used.
///
/// # Forms
///
/// `define!` chooses parser and default-help-repr independently:
///
/// | parser                     | default repr             | syntax                                               |
/// |----------------------------|--------------------------|------------------------------------------------------|
/// | `<T as FromStr>::from_str` | `format!("{}", default)` | `static X: T = default`                              |
/// | `<T as FromStr>::from_str` | `(repr)(default)`        | `static X: T = default, repr = repr`                 |
/// | `fn(&str) -> Result<T, _>` | `format!("{}", default)` | `static X: T = default, parse = parser`              |
/// | `fn(&str) -> Result<T, _>` | `(repr)(default)`        | `static X: T = default, parse = parser, repr = repr` |
///
/// Default-parser arms require `T: FromStr` with `T::Err: Display`.
/// Default-repr arms require `T: Display`. The custom-`repr` arms accept
/// any `Fn(T) -> Option<String>` — return `None` to suppress the
/// `[default: …]` block in `--help`. Doc comments on the static are
/// forwarded to the generated item (visible in rustdoc and, with the
/// `inventory` feature, in `--help`).
///
/// # Example — default parser, default repr
///
/// ```
/// enventory::define! {
///     /// Port to listen on
///     pub static MY_PORT: u16 = 8080
/// }
///
/// // Reads `MY_PORT` from env on first deref; falls back to 8080.
/// let _port: u16 = *MY_PORT;
/// ```
///
/// # Example — custom parser
///
/// Use when `T` doesn't impl `FromStr`, or when you want bespoke
/// behaviour (case-insensitive matching, alternative spellings, …):
///
/// ```
/// fn parse_lower(s: &str) -> Result<String, String> {
///     Ok(s.to_lowercase())
/// }
///
/// enventory::define! {
///     pub static MY_NAME: String = "world".into(), parse = parse_lower
/// }
///
/// println!("hello {}", *MY_NAME);
/// ```
///
/// # Example — custom repr
///
/// For types where `Display` of the default isn't useful (e.g. `Option`).
/// The closure receives the typed default and returns its
/// `--help`-visible string form:
///
/// ```
/// enventory::define! {
///     pub static OPT_PORT: Option<u16> = None,
///         parse = enventory::parse_some::<u16>,
///         repr = enventory::option_repr
/// }
///
/// let _v: Option<u16> = *OPT_PORT;
/// ```
///
/// # Feature interaction
///
/// Library crates depending on `enventory` with default features get just
/// the typed `Var<T, _>` static — the registration call expands to nothing.
/// When a downstream binary enables the `inventory` (or `clap`) feature,
/// cargo's feature unification reactivates the registration in every
/// transitive library that called `define!`. Same source code, two
/// deployment modes.
#[macro_export]
macro_rules! define {
    // Custom parser + custom repr
    (
        $(#[doc = $doc:literal])*
        $vis:vis static $name:ident : $ty:ty = $default:expr, parse = $parser:expr, repr = $repr:expr
    ) => {
        $(#[doc = $doc])*
        $vis static $name: $crate::Var<$ty, ::std::string::String> = $crate::Var::new(
            stringify!($name),
            $parser,
            || $default,
        );

        $crate::__register_entry!(
            $name,
            concat!($($doc, "\n",)*),
            || ($repr)($name.default_value())
        );
    };
    // Custom parser, default repr (uses Display)
    (
        $(#[doc = $doc:literal])*
        $vis:vis static $name:ident : $ty:ty = $default:expr, parse = $parser:expr
    ) => {
        $(#[doc = $doc])*
        $vis static $name: $crate::Var<$ty, ::std::string::String> = $crate::Var::new(
            stringify!($name),
            $parser,
            || $default,
        );

        $crate::__register_entry!(
            $name,
            concat!($($doc, "\n",)*),
            || ::std::option::Option::Some(::std::format!("{}", $name.default_value()))
        );
    };
    // Default parser (FromStr) + custom repr
    (
        $(#[doc = $doc:literal])*
        $vis:vis static $name:ident : $ty:ty = $default:expr, repr = $repr:expr
    ) => {
        $(#[doc = $doc])*
        $vis static $name: $crate::Var<$ty, <$ty as ::core::str::FromStr>::Err> = $crate::Var::new(
            stringify!($name),
            <$ty as ::core::str::FromStr>::from_str,
            || $default,
        );

        $crate::__register_entry!(
            $name,
            concat!($($doc, "\n",)*),
            || ($repr)($name.default_value())
        );
    };
    // Default parser (FromStr), default repr (uses Display)
    (
        $(#[doc = $doc:literal])*
        $vis:vis static $name:ident : $ty:ty = $default:expr
    ) => {
        $(#[doc = $doc])*
        $vis static $name: $crate::Var<$ty, <$ty as ::core::str::FromStr>::Err> = $crate::Var::new(
            stringify!($name),
            <$ty as ::core::str::FromStr>::from_str,
            || $default,
        );

        $crate::__register_entry!(
            $name,
            concat!($($doc, "\n",)*),
            || ::std::option::Option::Some(::std::format!("{}", $name.default_value()))
        );
    };
}

#[cfg(feature = "inventory")]
#[macro_export]
#[doc(hidden)]
macro_rules! __register_entry {
    ($name:ident, $desc:expr, $default_repr:expr) => {
        $crate::inventory::submit! {
            $crate::Item::new($name.key())
                .with_help($desc)
                .with_default_value($default_repr)
                .with_validator(|s: &::std::ffi::OsStr| $name.set_from_os_str(s)
                    .map_err(::core::convert::Into::into))
        }
    };
    ($name:ident, $desc:expr, $default_repr:expr, possible_values = $pv:expr) => {
        $crate::inventory::submit! {
            $crate::Item::new($name.key())
                .with_help($desc)
                .with_default_value($default_repr)
                .with_validator(|s: &::std::ffi::OsStr| $name.set_from_os_str(s)
                    .map_err(::core::convert::Into::into))
                .with_possible_values($pv)
        }
    };
}

#[cfg(not(feature = "inventory"))]
#[macro_export]
#[doc(hidden)]
macro_rules! __register_entry {
    ($name:ident, $desc:expr, $default_repr:expr) => {};
    ($name:ident, $desc:expr, $default_repr:expr, possible_values = $pv:expr) => {};
}

/// Like [`define!`], but for `bool` variables parsed with
/// [`parse_boolish`](crate::parse_boolish). The env var accepts `true` /
/// `false` / `yes` / `no` / `on` / `off` / `1` / `0` (case-insensitive)
/// instead of just `true` / `false`.
///
/// ```
/// enventory::define_boolish! {
///     /// Enable verbose logging
///     pub static VERBOSE: bool = false
/// }
/// # fn main() { let _ = *VERBOSE; }
/// ```
#[macro_export]
macro_rules! define_boolish {
    (
        $(#[doc = $doc:literal])*
        $vis:vis static $name:ident : bool = $default:expr
    ) => {
        $(#[doc = $doc])*
        $vis static $name: $crate::Var<bool, ::std::string::String> = $crate::Var::new(
            stringify!($name),
            $crate::parse_boolish,
            || $default,
        );

        $crate::__register_entry!(
            $name,
            concat!($($doc, "\n",)*),
            || ::std::option::Option::Some(::std::format!("{}", $name.default_value())),
            possible_values = &[
                $crate::PossibleValue::new("true"),
                $crate::PossibleValue::new("false"),
                $crate::PossibleValue::new("yes"),
                $crate::PossibleValue::new("no"),
                $crate::PossibleValue::new("on"),
                $crate::PossibleValue::new("off"),
                $crate::PossibleValue::new("1"),
                $crate::PossibleValue::new("0"),
            ]
        );
    };
}
