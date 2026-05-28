use std::collections::{HashMap, HashSet};
use std::{env, fmt};

use crate::Item;

/// A single environment variable that could not be parsed.
#[derive(Debug)]
pub struct ParseError {
    pub name: &'static str,
    pub type_name: &'static str,
    pub value: String,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}={:?}: failed to parse as {}: {}",
            self.name, self.value, self.type_name, self.message
        )
    }
}

impl std::error::Error for ParseError {}

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

/// Metadata for a registered environment variable.
///
/// Populated by the [`crate::define!`] macro and collected via the `inventory` crate.
#[derive(Debug)]
pub struct ItemEntry {
    pub name: &'static str,
    pub description: &'static str,
    pub type_name: &'static str,
    pub default_repr: &'static str,
    pub crate_name: &'static str,
    pub init: fn(Option<&str>) -> Result<(), ParseError>,
}

inventory::collect!(ItemEntry);

pub(crate) fn iter() -> impl Iterator<Item = &'static ItemEntry> {
    inventory::iter::<ItemEntry>.into_iter()
}

// TODO: Enable binaries to resolve conflicts
pub(crate) fn check_consistency() {
    let mut seen: HashMap<&str, &ItemEntry> = HashMap::new();
    for entry in iter() {
        if let Some(prev) = seen.get(entry.name) {
            if prev.type_name != entry.type_name
                || prev.description != entry.description
                || prev.default_repr != entry.default_repr
            {
                panic!(
                    "conflicting registrations for environment variable '{}': \
                     registered by '{}' and '{}' with different configurations",
                    entry.name, prev.crate_name, entry.crate_name
                );
            }
        } else {
            seen.insert(entry.name, entry);
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
        if !seen.insert(entry.name) {
            continue;
        }
        if let Err(e) = (entry.init)(None) {
            errors.push(e);
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(ValidationErrors { errors })
    }
}

impl<T> Item<T> {
    #[doc(hidden)]
    pub fn init(&self, override_value: Option<&str>) -> Result<(), ParseError> {
        let raw = match override_value {
            Some(val) => Some(val.to_owned()),
            None => match env::var(self.name) {
                Ok(val) => Some(val),
                Err(env::VarError::NotPresent) => None,
                Err(env::VarError::NotUnicode(_)) => {
                    return Err(ParseError {
                        name: self.name,
                        type_name: self.type_name,
                        value: "<invalid unicode>".to_owned(),
                        message: "environment variable contains invalid unicode".to_owned(),
                    });
                }
            },
        };

        match raw {
            Some(val) => match (self.parse)(&val) {
                Ok(parsed) => {
                    let _ = self.value.set(parsed);
                    Ok(())
                }
                Err(message) => Err(ParseError {
                    name: self.name,
                    type_name: self.type_name,
                    value: val,
                    message,
                }),
            },
            None => {
                let _ = self.value.set((self.default)());
                Ok(())
            }
        }
    }
}
