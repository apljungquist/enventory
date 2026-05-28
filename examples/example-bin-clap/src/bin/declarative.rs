use clap::Parser;

#[derive(Parser)]
#[command(
    bin_name = "example-bin",
    about = "Example binary demonstrating enventory"
)]
struct Cli {
    #[command(flatten)]
    env: enventory::EnvArgs,
}

fn main() {
    let _cli = Cli::parse();
    example_lib::do_work();
    example_lib_no_macros::do_work();
}
