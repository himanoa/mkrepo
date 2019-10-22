extern crate failure;
extern crate toml;
extern crate serde;
extern crate serde_derive;

pub mod makerepo {
    use serde_derive::Deserialize;
    use failure::Error;
    use std::fs::{create_dir_all, read_to_string};

    #[derive(Debug, PartialEq)]
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

    pub fn build_commands<'a>(config: Config, name: Option<&str>, service_name: Option<&str>, repository_name: &str, first_commit_message: Option<&str>) -> Vec<CommandType<'a>> {
        unimplemented!()
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        pub fn build_commands_return_to_create_directory_and_initialize_git() {
            let c = Config {
                service: "github.com".to_string(),
                name: Some("himanoa".to_string()),
                ghq_root: Some("~/src".to_string())
            };
            assert_eq!(build_commands(c, None, None, "mkrepo", Some("Initial commit")), vec![CommandType::CreateDirectory {
                path: "~/src/github.com/himanoa/mkrepo"
            }, CommandType::InitializeGit {
                first_commit_message: "Initial commit",
                path: "~/src/github.com/himanoa/mkrepo"
            }]);
        }
    }
}
