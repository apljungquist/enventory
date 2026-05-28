use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::{env, fmt};

use enventory_core::{Item, PossibleValue, SetError, iter};

/// One or more environment variables that could not be parsed.
///
/// Returned by [`set_all_from_env`] when at least one registered variable
/// has an invalid value in the environment.
#[derive(Debug)]
pub struct ValidationErrors {
    /// One [`SetError`] per registered variable whose validator rejected
    /// the value found in the environment. Empty when [`set_all_from_env`]
    /// returned `Ok(())`.
    pub errors: Vec<SetError>,
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Environment variable validation failed:")?;
        for error in &self.errors {
            writeln!(f, "  - {error}")?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationErrors {}

// TODO: Enable binaries to resolve conflicts
pub(crate) fn check_consistency() {
    let mut seen: HashMap<&str, &Item> = HashMap::new();
    for entry in iter() {
        let name = entry.key();
        if let Some(prev) = seen.get(name) {
            if prev.help() != entry.help()
                || prev.default_value() != entry.default_value()
                || !possible_values_equivalent(prev.possible_values(), entry.possible_values())
            {
                panic!(
                    "conflicting registrations for environment variable '{name}': \
                     two or more libraries registered it with different configurations"
                );
            }
        } else {
            seen.insert(name, entry);
        }
    }
}

/// Compare two `possible_values` listings by their observable fields.
/// Neither `Option` nor `PossibleValue` implement `PartialEq` — `Option`
/// to keep `None` ("unrestricted") distinct from `Some(&[])` ("explicit
/// empty list") even when an opinionated `PartialEq` would conflate them,
/// `PossibleValue` to keep adding a future field (e.g. `hide`) from
/// silently changing downstream equality. This helper is updated when a
/// new field gets added.
fn possible_values_equivalent(a: Option<&[PossibleValue]>, b: Option<&[PossibleValue]>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => {
            a.len() == b.len()
                && a.iter()
                    .zip(b)
                    .all(|(x, y)| x.name() == y.name() && x.help() == y.help())
        }
        _ => false,
    }
}

/// Eagerly parses and caches every registered environment variable.
///
/// Returns `Ok(())` if all variables are valid or absent (in which case
/// the registered default representation, if any, is used). Returns
/// `Err` collecting every parse failure.
pub fn set_all_from_env() -> Result<(), ValidationErrors> {
    check_consistency();
    let mut seen = HashSet::new();
    let mut errors = Vec::new();
    for entry in iter() {
        let name = entry.key();
        if !seen.insert(name) {
            continue;
        }
        let result = match env::var_os(name) {
            Some(v) => entry.validate(&v),
            None => match entry.default_value() {
                Some(repr) => entry.validate(OsStr::new(&repr)),
                // No env var, no default value — leave the cache untouched.
                // `Var::deref` will lazily resolve to the typed default on
                // first access.
                None => Ok(()),
            },
        };
        if let Err(e) = result {
            errors.push(e);
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(ValidationErrors { errors })
    }
}
