use std::collections::HashSet;
use std::ffi::{OsStr, OsString};

use clap::builder::{OsStringValueParser, PossibleValuesParser, TypedValueParser};
use enventory_core::SetError;

use crate::{check_consistency, iter};

/// Builds the `value_parser` for `entry`. Two cases — the cache write
/// always goes through `entry.set_from_os_str`:
///
///   * No `possible_values`: clap's `OsStringValueParser` passes raw input
///     through to `set_from_os_str`; matches stored as `OsString`.
///   * With `possible_values`: clap's `PossibleValuesParser` is `String`-typed
///     (the listed values are UTF-8 literals), so input is UTF-8-validated
///     and matched against the list before being re-wrapped as `OsString`
///     and handed to `set_from_os_str`.
fn build_value_parser(entry: &'static enventory_core::Item) -> clap::builder::ValueParser {
    let possible_values: Vec<clap::builder::PossibleValue> = entry
        .possible_values()
        .unwrap_or_default()
        .iter()
        .map(|pv| {
            let mut cpv = clap::builder::PossibleValue::new(pv.name());
            if let Some(h) = pv.help() {
                cpv = cpv.help(h);
            }
            cpv
        })
        .collect();

    if possible_values.is_empty() {
        OsStringValueParser::new()
            .try_map(move |s: OsString| -> Result<OsString, SetError> {
                entry.validate(&s)?;
                Ok(s)
            })
            .into()
    } else {
        PossibleValuesParser::new(possible_values)
            .try_map(move |s: String| -> Result<OsString, SetError> {
                let os = OsString::from(s);
                entry.validate(&os)?;
                Ok(os)
            })
            .into()
    }
}

/// Strips the leading space that rustdoc adds to each `///` line.
fn normalize_doc(s: &str) -> String {
    s.lines()
        .map(|l| l.strip_prefix(' ').unwrap_or(l))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_owned()
}

/// Builds one [`clap::Arg`] per registered environment variable.
///
/// Most binaries use [`EnvArgs`] via `#[command(flatten)]` and never call
/// `args()` directly — this is the escape hatch for builders that
/// hand-construct a [`clap::Command`] rather than going through the
/// `derive` API:
///
/// ```
/// let cmd = clap::Command::new("app").args(enventory::args());
/// # let _ = cmd;
/// ```
///
/// Each arg has a long flag (lowercased + hyphens, e.g. `MY_PORT` →
/// `--my-port`), an env-var fallback matching the registered name, the
/// doc-comment as help text, and a `value_parser` that — on successful
/// parse — writes through to the corresponding [`Var<T>`](crate::Var)
/// typed cache.
pub fn args() -> Vec<clap::Arg> {
    check_consistency();
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for entry in iter() {
        let name = entry.key();
        if !seen.insert(name) {
            continue;
        }

        let long: &'static str = Box::leak(name.to_lowercase().replace('_', "-").into_boxed_str());
        let mut arg = clap::Arg::new(name).long(long).env(name);

        if let Some(default_str) = entry.default_value() {
            let leaked: &'static str = Box::leak(default_str.into_boxed_str());
            arg = arg.default_value(leaked);
        }

        if let Some(h) = entry.help() {
            let desc = normalize_doc(h);
            if !desc.is_empty() {
                let leaked: &'static str = Box::leak(desc.into_boxed_str());
                arg = arg.help(leaked);
            }
        }

        arg = arg.value_parser(build_value_parser(entry));

        result.push(arg);
    }
    result.sort_by(|a, b| a.get_id().as_str().cmp(b.get_id().as_str()));
    result
}

/// Iterates registered items, reads each entry's value from `matches`,
/// and runs the registered validator — populating the typed
/// [`Var<T>`](crate::Var) caches the same way
/// [`set_all_from_env`](crate::set_all_from_env) does, but with
/// `ArgMatches` as the source instead of `env::var_os`.
///
/// Normally a no-op: when args are built via [`args`] (or `EnvArgs`'s
/// `augment_args`), clap already ran the validator during `get_matches`,
/// so the cache is already populated by the time this is called. The
/// function is the fallback for binaries that hand-build a
/// [`clap::Command`] *without* `args()` — useful, but lossy: it only
/// recognizes `String` and `OsString` storage types, so user-typed args
/// (e.g. `value_parser!(u16)`) sharing an env-var id won't be picked up.
pub fn set_all_from_matches(matches: &clap::ArgMatches) -> Result<(), clap::Error> {
    let mut seen = HashSet::new();
    for entry in iter() {
        let name = entry.key();
        if !seen.insert(name) {
            continue;
        }
        // Detect storage type. Args built by `args()` store `String` (or
        // `OsString` when `accepts_invalid_utf8`). Either Ok(_) is fine.
        let has_string = matches.try_get_one::<String>(name).is_ok();
        let has_os = matches.try_get_one::<OsString>(name).is_ok();
        if !has_string && !has_os {
            // Arg wasn't in the command (e.g. binary stripped it / shadows
            // it with its own flag).
            continue;
        }
        // Skip if no value was provided (optional arg with no default).
        if matches.value_source(name).is_none() {
            continue;
        }
        // Skip default values — the value_parser already validated them
        // during get_matches(), so the cache is populated.
        if matches.value_source(name) == Some(clap::parser::ValueSource::DefaultValue) {
            continue;
        }
        // With our `value_parser` on the arg, these calls are normally a
        // no-op (already cached). They serve as a fallback for commands not
        // built with `args()`. Errors here propagate as `ValueValidation`
        // so callers don't silently end up with a stale/default cache.
        let result = if has_string {
            matches
                .get_one::<String>(name)
                .map(|v| entry.validate(OsStr::new(v)))
        } else {
            matches.get_one::<OsString>(name).map(|v| entry.validate(v))
        };
        if let Some(Err(e)) = result {
            return Err(clap::Error::raw(
                clap::error::ErrorKind::ValueValidation,
                format!("{e}\n"),
            ));
        }
    }
    Ok(())
}

/// A unit struct that implements [`clap::Args`], designed for
/// `#[command(flatten)]` on a clap-derived `Parser`.
///
/// Flattening `EnvArgs` contributes one `--<long>` flag per env var
/// registered via [`define!`](crate::define) (or `inventory::submit!`),
/// each with env-var fallback, the doc-comment as help text, and the
/// registered default rendered as `[default: …]`. Parsing populates the
/// per-variable [`Var<T>`](crate::Var) cache as a side effect, so library
/// code reading `*MY_VAR` after `Cli::parse` sees whatever the user
/// supplied (or the registered default).
///
/// # Example
///
/// ```
/// use clap::Parser;
///
/// enventory::define! {
///     /// Port to listen on
///     pub static MY_PORT: u16 = 8080
/// }
///
/// #[derive(Parser)]
/// struct Cli {
///     #[command(flatten)]
///     env: enventory::EnvArgs,
/// }
///
/// let _ = Cli::try_parse_from(["app"]).unwrap();
/// println!("listening on {}", *MY_PORT);
/// ```
#[derive(Default)]
pub struct EnvArgs;

impl clap::FromArgMatches for EnvArgs {
    fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
        set_all_from_matches(matches)?;
        Ok(EnvArgs)
    }

    fn update_from_arg_matches(&mut self, matches: &clap::ArgMatches) -> Result<(), clap::Error> {
        set_all_from_matches(matches)
    }
}

impl clap::Args for EnvArgs {
    fn augment_args(cmd: clap::Command) -> clap::Command {
        cmd.args(args())
    }

    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        cmd.args(args())
    }
}
