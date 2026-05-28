fn main() {
    let cmd = clap::Command::new("example-bin")
        .bin_name("example-bin")
        .about("Example binary demonstrating enventory")
        .args(enventory::args());

    let matches = cmd.get_matches();
    enventory::set_all_from_matches(&matches).unwrap_or_else(|e| e.exit());

    example_lib::do_work();
    example_lib_no_macros::do_work();
}
