use std::num::ParseIntError;
use std::str::FromStr;

use enventory::{Item, Var, inventory};

// Essentially a LazyLock with a few more options
pub static PORT: Var<u16, ParseIntError> = Var::new("PORT", u16::from_str, || 8080);

// No additional magic on top of inventory. Use the builder so adding future
// metadata fields stays a non-breaking change for this call site.
inventory::submit! {
    Item::new(PORT.key())
        .with_help("Port to listen on")
        .with_default_value(|| Some(PORT.default_value().to_string()))
        .with_validator(|s| PORT.set_from_os_str(s).map_err(Into::into))
}

pub fn do_work() {
    println!("Listening on port {}", *PORT);
}
