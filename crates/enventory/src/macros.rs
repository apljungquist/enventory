#[macro_export]
macro_rules! define {
    // With doc, default, and custom parser
    (
        #[doc = $doc:literal]
        $(#[$rest:meta])*
        $vis:vis static $name:ident : $ty:ty = $default:expr, parse = $parser:expr
    ) => {
        $(#[$rest])*
        $vis static $name: $crate::Item<$ty> = $crate::Item::new(
            stringify!($name),
            stringify!($ty),
            || $default,
            $parser,
        );

        $crate::__register_entry!(
            $name,
            $doc,
            stringify!($ty),
            stringify!($default)
        );
    };
    // Without doc, with default and custom parser
    (
        $vis:vis static $name:ident : $ty:ty = $default:expr, parse = $parser:expr
    ) => {
        $vis static $name: $crate::Item<$ty> = $crate::Item::new(
            stringify!($name),
            stringify!($ty),
            || $default,
            $parser,
        );

        $crate::__register_entry!(
            $name,
            "",
            stringify!($ty),
            stringify!($default)
        );
    };
    // With doc and default (uses FromStr)
    (
        #[doc = $doc:literal]
        $(#[$rest:meta])*
        $vis:vis static $name:ident : $ty:ty = $default:expr
    ) => {
        $(#[$rest])*
        $vis static $name: $crate::Item<$ty> = $crate::Item::new(
            stringify!($name),
            stringify!($ty),
            || $default,
            $crate::parse_from_str,
        );

        $crate::__register_entry!(
            $name,
            $doc,
            stringify!($ty),
            stringify!($default)
        );
    };
    // Without doc, with default (uses FromStr)
    (
        $vis:vis static $name:ident : $ty:ty = $default:expr
    ) => {
        $vis static $name: $crate::Item<$ty> = $crate::Item::new(
            stringify!($name),
            stringify!($ty),
            || $default,
            $crate::parse_from_str,
        );

        $crate::__register_entry!(
            $name,
            "",
            stringify!($ty),
            stringify!($default)
        );
    };
}

#[cfg(feature = "inventory")]
#[macro_export]
#[doc(hidden)]
macro_rules! __register_entry {
    ($name:ident, $doc:expr, $ty_str:expr, $default_str:expr) => {
        $crate::inventory::submit! {
            $crate::ItemEntry {
                name: stringify!($name),
                description: $doc,
                type_name: $ty_str,
                default_repr: $default_str,
                crate_name: env!("CARGO_PKG_NAME"),
                init: |override_val| $name.init(override_val),
            }
        }
    };
}

#[cfg(not(feature = "inventory"))]
#[macro_export]
#[doc(hidden)]
macro_rules! __register_entry {
    ($name:ident, $doc:expr, $ty_str:expr, $default_str:expr) => {};
}

/// Like [`define!`], but for `bool` variables parsed with [`crate::parse_boolish`].
#[macro_export]
macro_rules! define_boolish {
    (
        #[doc = $doc:literal]
        $(#[$rest:meta])*
        $vis:vis static $name:ident : bool = $default:expr
    ) => {
        $crate::define!(
            #[doc = $doc]
            $(#[$rest])*
            $vis static $name: bool = $default, parse = $crate::parse_boolish
        );
    };
    (
        $vis:vis static $name:ident : bool = $default:expr
    ) => {
        $crate::define!(
            $vis static $name: bool = $default, parse = $crate::parse_boolish
        );
    };
}
