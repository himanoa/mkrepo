extern crate clap;
use clap::{App, Arg};
mod lib;
use lib::makerepo::{CommandType, DryRunExecutor, Executor};

const VERSION: &'static str = "0.0.1";
fn main() {
    App::new("Make project directory for ghq style.")
        .version(VERSION)
        .arg(
            Arg::with_name("project_name")
                .help("Repository name")
                .required(true)
                .index(1),
        )
        .get_matches();
    let executor = DryRunExecutor::new();
    match executor.execute(vec![
        CommandType::CreateDirectory {
            path: "himanoa".to_string(),
        },
        CommandType::InitializeGit {
            first_commit_message: "Initial commit".to_string(),
            path: "himanoa".to_string(),
        },
    ]) {
        Ok(()) => (),
        Err(_) => (),
    };
}
