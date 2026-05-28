use std::collections::HashSet;

use crate::{check_consistency, iter};

/// Builds a [`clap::Arg`] for each registered environment variable.
pub fn args() -> Vec<clap::Arg> {
    check_consistency();
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for entry in iter() {
        if !seen.insert(entry.name) {
            continue;
        }

        let long: &'static str =
            Box::leak(entry.name.to_lowercase().replace('_', "-").into_boxed_str());
        let mut arg = clap::Arg::new(entry.name).long(long).env(entry.name);

        arg = arg.default_value(entry.default_repr);

        if !entry.description.is_empty() {
            arg = arg.help(entry.description.trim());
        }

        result.push(arg);
    }
    result.sort_by(|a, b| a.get_id().as_str().cmp(b.get_id().as_str()));
    result
}

/// Validates and caches environment variables from parsed clap matches.
pub fn apply_matches(matches: &clap::ArgMatches) -> Result<(), clap::Error> {
    apply_matches_impl(matches, None)
}

/// Like [`apply_matches`], but formats errors using the given [`clap::Command`].
pub fn apply_matches_for(
    cmd: &mut clap::Command,
    matches: &clap::ArgMatches,
) -> Result<(), clap::Error> {
    apply_matches_impl(matches, Some(cmd))
}

fn apply_matches_impl(
    matches: &clap::ArgMatches,
    cmd: Option<&mut clap::Command>,
) -> Result<(), clap::Error> {
    let mut seen = HashSet::new();
    for entry in iter() {
        if !seen.insert(entry.name) {
            continue;
        }
        // Skip default values - the stringify! representation may not
        // round-trip through the parser (e.g. "Seconds(30)"). The Item's
        // own typed default is used via Deref instead.
        let source = matches.value_source(entry.name);
        if source == Some(clap::parser::ValueSource::DefaultValue) {
            continue;
        }
        if let Some(value) = matches.get_one::<String>(entry.name) {
            if let Err(e) = (entry.init)(Some(value)) {
                let err =
                    clap::Error::raw(clap::error::ErrorKind::ValueValidation, format!("{e}\n"));
                return match cmd {
                    Some(cmd) => Err(err.format(cmd)),
                    None => Err(err),
                };
            }
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
