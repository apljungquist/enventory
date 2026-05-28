enventory::define!(
    /// Port to listen on
    pub static MYHTTP_PORT: u16 = 8080
);

pub fn start_server() {
    let port = *MYHTTP_PORT;
    println!("Listening on port {port}");
}
