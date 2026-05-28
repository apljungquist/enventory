fn main() {
    enventory::set_all_from_env().expect("validation failed");
    example_lib::do_work();
    example_lib_no_macros::do_work();
}
