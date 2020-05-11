use clap::{App, Arg};
use git2::Config as GitConfig;
use mkrepo::makerepo::{
    build_commands, load_git_config, DefaultExecutor, DryRunExecutor, Executor,
};
use std::process::exit;

fn main() {
    let matchers = App::new("Make project directory for ghq style.")
        .version(env!("CARGO_PKG_VERSION"))
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
                .takes_value(true)
                .long("author"),
        )
        .arg(
            Arg::with_name("service")
                .help("service name")
                .required(false)
                .short("s")
                .takes_value(true)
                .long("service"),
        )
        .arg(
            Arg::with_name("first_commit_message")
                .help("first_commit_message name")
                .required(false)
                .takes_value(true)
                .short("m"),
        )
        .arg(
            Arg::with_name("dry_run")
                .help("dru run")
                .required(false)
                .short("d"),
        )
        .arg(
            Arg::with_name("project_name")
                .help("The name of the project template to be expanded")
                .required(false)
                .takes_value(true)
                .short("p"),
        )
        .get_matches();
    let config = match load_git_config(GitConfig::open_default().unwrap()) {
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
        matchers.value_of("project_name"),
    ) {
        Ok(commands) => {
            if matchers.is_present("dry_run") {
                let executor = DryRunExecutor::new();
                match executor.execute(commands) {
                    Ok(()) => (),
                    Err(e) => {
                        eprintln!("{}", e);
                        exit(1)
                    }
                };
            } else {
                let executor = DefaultExecutor::new();
                match executor.execute(commands) {
                    Ok(()) => (),
                    Err(e) => {
                        eprintln!("{}", e);
                        exit(1)
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
