//! Bridges [`crate::var_core::Var`] to the [`enventory_core::VarRef`] trait.
//!
//! The trait, [`enventory_core::Item`] struct, and `inventory::collect!`
//! invocation live in the separate `enventory-core` crate so user crates can
//! register variables without pulling in any of the higher-level machinery.
//! The implementation stays here because `Var` is defined in this crate.

use std::any;
use std::fmt::Display;

use enventory_core::{SetError, VarRef};

use crate::var_core::{ParseError, Var};

impl<T, E> VarRef for Var<T, E>
where
    T: Send + Sync + 'static,
    E: Display + 'static,
{
    fn key(&self) -> &'static str {
        self.key
    }
    fn type_name(&self) -> &'static str {
        any::type_name::<T>()
    }
    fn set_from_str(&self, s: &str) -> Result<(), SetError> {
        Var::set_from_str(self, s).map_err(parse_error_to_set_error)
    }
    fn set_from_env(&self) -> Result<(), SetError> {
        Var::set_from_env(self).map_err(parse_error_to_set_error)
    }
}

fn parse_error_to_set_error(e: ParseError) -> SetError {
    SetError::new(e.name(), e.value().to_owned(), e.message().to_owned())
}
