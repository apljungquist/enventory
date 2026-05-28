use std::num::ParseIntError;
use std::str::FromStr;

use enventory::{Item, Var, inventory};

// Essentially a LazyLock with a few more options
pub static PORT: Var<u16, ParseIntError> = Var::new("PORT", u16::from_str, || 8080);

// No additional magic on top of inventory
inventory::submit! {
    Item {
        item: &PORT,
        description: "Port to listen on",
        crate_name: env!("CARGO_PKG_NAME"),
        default_repr: || Some(8080.to_string()),
    }
}

pub fn do_work() {
    println!("Listening on port {}", *PORT);
}
