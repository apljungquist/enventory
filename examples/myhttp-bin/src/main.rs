use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    env: enventory::EnvArgs,
}

fn main() {
    let _cli = Cli::parse();
    myhttp::start_server();
}
