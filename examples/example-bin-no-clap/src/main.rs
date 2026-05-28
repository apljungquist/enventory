fn main() {
    enventory::validate_all().expect("validation failed");
    example_lib::do_work();
}
