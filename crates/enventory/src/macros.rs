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
            $crate::Item {
                item: &$name,
                description: $desc,
                crate_name: env!("CARGO_PKG_NAME"),
                default_repr: $default_repr,
            }
        }
    };
}

#[cfg(not(feature = "inventory"))]
#[macro_export]
#[doc(hidden)]
macro_rules! __register_entry {
    ($name:ident, $desc:expr, $default_repr:expr) => {};
}

/// Like [`define!`], but for `bool` variables parsed with [`crate::parse_boolish`].
#[macro_export]
macro_rules! define_boolish {
    (
        $(#[doc = $doc:literal])*
        $vis:vis static $name:ident : bool = $default:expr
    ) => {
        $crate::define! {
            $(#[doc = $doc])*
            $vis static $name: bool = $default, parse = $crate::parse_boolish
        }
    };
}
