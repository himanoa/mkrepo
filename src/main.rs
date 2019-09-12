extern crate clap;
use clap::App;

const VERSION: &'static str = "0.0.1";
fn main() {
    App::new("Make project directory for ghq style.")
        .version(VERSION)
        .get_matches();
}
