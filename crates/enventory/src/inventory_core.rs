//! Minimum machinery for cross-crate coordination: the [`Item`] struct plus the
//! `inventory::collect!` invocation. Library crates that register variables only
//! need this module plus [`crate::var_core`].

use std::any;
use std::fmt::Display;

use crate::var_core::{ParseError, Var};

mod sealed {
    pub trait Sealed {}
}

/// Type-erased view of a [`Var`] that the inventory layer can dispatch through
/// without knowing the concrete `T` or `E`.
///
/// Sealed — only `Var<T, E>` impls this. The trait is `pub` solely so that
/// `&'static dyn VarRef` is nameable at struct-literal sites.
#[doc(hidden)]
pub trait VarRef: sealed::Sealed + Sync {
    fn key(&self) -> &'static str;
    fn type_name(&self) -> &'static str;
    fn set_from_str(&self, s: &str) -> Result<(), ParseError>;
    fn set_from_env(&self) -> Result<(), ParseError>;
}

impl<T, E> sealed::Sealed for Var<T, E>
where
    T: Send + Sync + 'static,
    E: Display + 'static,
{
}

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
    fn set_from_str(&self, s: &str) -> Result<(), ParseError> {
        Var::set_from_str(self, s)
    }
    fn set_from_env(&self) -> Result<(), ParseError> {
        Var::set_from_env(self)
    }
}

/// Metadata for a registered environment variable.
///
/// Populated by the [`crate::define!`] macro and collected via the `inventory` crate.
pub struct Item {
    pub item: &'static dyn VarRef,
    pub description: &'static str,
    pub crate_name: &'static str,
    pub default_repr: fn() -> Option<String>,
}

inventory::collect!(Item);

pub(crate) fn iter() -> impl Iterator<Item = &'static Item> {
    inventory::iter::<Item>.into_iter()
}
