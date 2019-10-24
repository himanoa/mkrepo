extern crate clap;
use clap::{App, Arg};
mod lib;
use lib::makerepo::{build_commands, load_git_config, DefaultExecutor, DryRunExecutor, Executor};

const VERSION: &'static str = "0.0.1";
fn main() {
    let matchers = App::new("Make project directory for ghq style.")
        .version(VERSION)
        .arg(
            Arg::with_name("repository")
                .help("Repository name")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("author")
                .help("repository author name")
                .required(false)
                .short("a")
                .long("author"),
        )
        .arg(
            Arg::with_name("service")
                .help("service name")
                .required(false)
                .short("s")
                .long("service"),
        )
        .arg(
            Arg::with_name("first_commit_message")
                .help("first_commit_message name")
                .required(false)
                .short("m"),
        )
        .arg(
            Arg::with_name("dry_run")
                .help("dru run")
                .required(false)
                .short("d"),
        )
        .get_matches();
    let config = match load_git_config() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{}", e);
            panic!()
        }
    };

    match build_commands(
        config,
        matchers.value_of("author"),
        matchers.value_of("service"),
        matchers.value_of("repository").unwrap(),
        matchers.value_of("first_commit_message"),
    ) {
        Ok(commands) => {
            if let Some(_) = matchers.value_of("dry_run") {
                let executor = DryRunExecutor::new();
                match executor.execute(commands) {
                    Ok(()) => (),
                    Err(e) => {
                        eprintln!("{}", e);
                        panic!()
                    }
                };
            } else {
                let executor = DefaultExecutor::new();
                match executor.execute(commands) {
                    Ok(()) => (),
                    Err(e) => {
                        eprintln!("{}", e);
                        panic!()
                    }
                };
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            panic!()
        }
    }
}
