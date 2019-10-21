extern crate failure;
extern crate toml;
extern crate serde;
extern crate serde_derive;

pub mod makerepo {
    use serde_derive::Deserialize;
    use failure::Error;
    use std::fs::{create_dir_all, read_to_string};

    #[derive(Debug)]
    pub enum CommandType<'a> {
        CreateDirectory {
            path: &'a str
        },
        InitializeGit {
            first_commit_message: &'a str,
            path: &'a str
        },
    }

    pub trait Executor {
        fn execute(&self, commands: Vec<CommandType>) -> Result<(), Error>;
    }

    pub struct DryRunExecutor {}

    impl DryRunExecutor {
        pub fn new() -> DryRunExecutor {
            DryRunExecutor {}
        }
    }

    impl Executor for DryRunExecutor {
        fn execute(&self, commands: Vec<CommandType>) -> Result<(), Error> {
            for command in commands {
                match command {
                    CommandType::CreateDirectory {
                        path
                    } => println!("CreateDirectory: {}", path),
                    CommandType::InitializeGit {
                        first_commit_message, path
                    } => println!("InitializeGit {} {}", first_commit_message, path),
                }
            }
            Ok(())
        }
    }

    pub fn create_directory(path: &str) -> Result<(), std::io::Error> {
        create_dir_all(path)
    }

    pub fn initialize_git(first_commit_message: &str, path: &str) -> Result<(), Error> {
        unimplemented!()
    }

    pub struct DefaultExecutor {}

    impl Executor for DefaultExecutor  {
        fn execute(&self, commands: Vec<CommandType>) -> Result<(), Error> {
            for command in commands {
                match command {
                    CommandType::CreateDirectory {
                        path
                    } => create_directory(path)?,
                    CommandType::InitializeGit {
                        first_commit_message, path
                    } => initialize_git(first_commit_message, path)?
                };
            };
            Ok(())
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct Config {
        service: String,
        name: Option<String>,
        ghq_root: Option<String>,
    }

    #[derive(Deserialize, Debug)]
    struct GitConfig {
        mkrepo: MkrepoConfig
    }

    #[derive(Deserialize, Debug)]
    struct MkrepoConfig {
        service: String,
        name: String,
        root: String
    }

    pub fn load_git_config() -> Result<Config, Error> {
        let git_config = read_to_string("~/.gitconfig")?;
        let config: GitConfig = toml::from_str(&git_config).unwrap();
        Ok(Config {
            service: config.mkrepo.service,
            name: Some(config.mkrepo.name),
            ghq_root: Some(config.mkrepo.root),
        })
    }
}
