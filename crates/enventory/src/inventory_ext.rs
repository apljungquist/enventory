//! Eager validation and consistency checks built on top of the `enventory-core`
//! crate. Binaries that want startup-time validation use this module; library
//! crates that only register variables do not.

use std::collections::{HashMap, HashSet};
use std::fmt;

use enventory_core::{Item, iter};

use crate::var_core::ParseError;

/// One or more environment variables that could not be parsed.
///
/// Returned by [`validate_all`] when at least one registered variable
/// has an invalid value in the environment.
#[derive(Debug)]
pub struct ValidationErrors {
    pub errors: Vec<ParseError>,
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
        let name = entry.item.key();
        if let Some(prev) = seen.get(name) {
            if prev.item.type_name() != entry.item.type_name()
                || prev.description != entry.description
                || (prev.default_repr)() != (entry.default_repr)()
            {
                panic!(
                    "conflicting registrations for environment variable '{}': \
                     registered by '{}' and '{}' with different configurations",
                    name, prev.crate_name, entry.crate_name
                );
            }
        } else {
            seen.insert(name, entry);
        }
    }
}

/// Eagerly parses and caches every registered environment variable.
///
/// Returns `Ok(())` if all variables are valid or absent (in which case
/// their defaults are used). Returns `Err` collecting every parse failure.
pub fn validate_all() -> Result<(), ValidationErrors> {
    check_consistency();
    let mut seen = HashSet::new();
    let mut errors = Vec::new();
    for entry in iter() {
        let name = entry.item.key();
        if !seen.insert(name) {
            continue;
        }
        if let Err(e) = entry.item.set_from_env() {
            errors.push(ParseError::new(
                e.name(),
                e.value().to_owned(),
                e.message().to_owned(),
            ));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(ValidationErrors { errors })
    }
}
