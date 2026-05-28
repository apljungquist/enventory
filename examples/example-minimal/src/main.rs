//! The minimal end-to-end example from the crate docs, reproduced as a
//! runnable binary. If you change anything here, mirror the change in the
//! `# Example` section of `crates/enventory/src/lib.rs` so the documented
//! `--help` output stays in sync with what this binary actually prints
//! (see `tests/cli.rs`).

enventory::define! {
    /// Port to listen on
    pub static MY_PORT: u16 = 8080
}

fn serve() {
    println!("Listening on port {}", *MY_PORT)
}

use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    env: enventory::EnvArgs,
}

fn main() {
    let _cli = Cli::parse();
    serve();
}
