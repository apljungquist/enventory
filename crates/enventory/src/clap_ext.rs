use std::collections::HashSet;
use std::fmt;

use crate::{check_consistency, iter};

#[derive(Debug, Clone)]
struct ValueParseError(String);

impl fmt::Display for ValueParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for ValueParseError {}

/// Strips the leading space that rustdoc adds to each `///` line.
fn normalize_doc(s: &str) -> String {
    s.lines()
        .map(|l| l.strip_prefix(' ').unwrap_or(l))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_owned()
}

/// Builds a [`clap::Arg`] for each registered environment variable.
pub fn args() -> Vec<clap::Arg> {
    check_consistency();
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for entry in iter() {
        let name = entry.item.key();
        if !seen.insert(name) {
            continue;
        }

        let long: &'static str = Box::leak(name.to_lowercase().replace('_', "-").into_boxed_str());
        let mut arg = clap::Arg::new(name).long(long).env(name);

        if let Some(default_str) = (entry.default_repr)() {
            let leaked: &'static str = Box::leak(default_str.into_boxed_str());
            arg = arg.default_value(leaked);
        }

        let desc = normalize_doc(entry.description);
        if !desc.is_empty() {
            let leaked: &'static str = Box::leak(desc.into_boxed_str());
            arg = arg.help(leaked);
        }

        let item = entry.item;
        arg = arg.value_parser(move |s: &str| -> Result<String, ValueParseError> {
            item.set_from_str(s)
                .map_err(|e| ValueParseError(e.message().to_owned()))?;
            Ok(s.to_owned())
        });

        result.push(arg);
    }
    result.sort_by(|a, b| a.get_id().as_str().cmp(b.get_id().as_str()));
    result
}

/// Validates and caches environment variables from parsed clap matches.
pub fn apply_matches(matches: &clap::ArgMatches) -> Result<(), clap::Error> {
    apply_matches_impl(matches)
}

/// Like [`apply_matches`], but formats errors using the given [`clap::Command`].
pub fn apply_matches_for(
    cmd: &mut clap::Command,
    matches: &clap::ArgMatches,
) -> Result<(), clap::Error> {
    // When args are built with `args()`, the value_parser already validated
    // and cached values during `get_matches()`. This call ensures caching
    // for any entries whose args were built without `args()`.
    let _ = cmd;
    apply_matches_impl(matches)
}

fn apply_matches_impl(matches: &clap::ArgMatches) -> Result<(), clap::Error> {
    let mut seen = HashSet::new();
    for entry in iter() {
        let name = entry.item.key();
        if !seen.insert(name) {
            continue;
        }
        // Skip entries whose args were filtered out (e.g. when the binary
        // defines its own CLI flag for the same variable).
        if matches.try_get_one::<String>(name).is_err() {
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
        if let Some(value) = matches.get_one::<String>(name) {
            // With value_parser on the arg, this is normally a no-op (already cached).
            // It serves as a fallback for commands not built with `args()`.
            let _ = entry.item.set_from_str(value);
        }
    }
    Ok(())
}

/// A unit struct that implements [`clap::Args`] for use with `#[command(flatten)]`.
///
/// Flattening this into a CLI struct adds all registered environment
/// variables as long options with env-var fallback and help text.
#[derive(Default)]
pub struct EnvArgs;

impl clap::FromArgMatches for EnvArgs {
    fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
        apply_matches(matches)?;
        Ok(EnvArgs)
    }

    fn update_from_arg_matches(&mut self, matches: &clap::ArgMatches) -> Result<(), clap::Error> {
        apply_matches(matches)
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
