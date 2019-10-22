extern crate failure;
extern crate serde;
extern crate serde_derive;
extern crate toml;

pub mod makerepo {
    use failure::Error;
    use serde_derive::Deserialize;
    use std::fs::{create_dir_all, read_to_string};
    use std::path::Path;
    use std::process::Command;

    #[derive(Debug, PartialEq)]
    pub enum CommandType {
        CreateDirectory {
            path: String,
        },
        InitializeGit {
            first_commit_message: String,
            path: String,
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
                    CommandType::CreateDirectory { path } => println!("CreateDirectory: {}", path),
                    CommandType::InitializeGit {
                        first_commit_message,
                        path,
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
        Command::new("git").arg("init").current_dir(path).output()?;
        Command::new("git")
            .args(&["commit", first_commit_message])
            .output();
        Ok(())
    }

    pub struct DefaultExecutor {}

    impl Executor for DefaultExecutor {
        fn execute(&self, commands: Vec<CommandType>) -> Result<(), Error> {
            for command in commands {
                match command {
                    CommandType::CreateDirectory { path } => create_directory(&path)?,
                    CommandType::InitializeGit {
                        first_commit_message,
                        path,
                    } => initialize_git(&first_commit_message, &path)?,
                };
            }
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
        mkrepo: MkrepoConfig,
    }

    #[derive(Deserialize, Debug)]
    struct MkrepoConfig {
        service: String,
        name: String,
        root: String,
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

    pub fn build_commands<'a>(
        config: Config,
        author: Option<&'a str>,
        service_name: Option<&'a str>,
        repository_name: &'a str,
        first_commit_message: Option<&'a str>,
    ) -> Result<std::vec::Vec<CommandType>, Error> {
        let parent_path = config.ghq_root.unwrap();
        let config_authro_name = config.name.unwrap();
        let service = match service_name {
            Some(n) => n,
            None => config.service.as_ref(),
        };
        let repository_author = match author {
            Some(n) => n,
            None => config_authro_name.as_ref(),
        };
        let repository_path = Path::new(&parent_path)
            .join(service)
            .join(repository_author)
            .join(repository_name);

        Ok(vec![
            CommandType::CreateDirectory {
                path: String::from(repository_path.to_str().unwrap()),
            },
            CommandType::InitializeGit {
                path: String::from(repository_path.to_str().unwrap()),
                first_commit_message: String::from(match first_commit_message {
                    Some(x) => x,
                    None => "Initial commit",
                }),
            },
        ])
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        pub fn build_commands_return_to_create_directory_and_initialize_git() {
            let c = Config {
                service: "github.com".to_string(),
                name: Some("himanoa".to_string()),
                ghq_root: Some("~/src".to_string()),
            };
            assert_eq!(
                build_commands(c, None, None, "mkrepo", Some("Initial commit")).unwrap(),
                vec![
                    CommandType::CreateDirectory {
                        path: String::from("~/src/github.com/himanoa/mkrepo")
                    },
                    CommandType::InitializeGit {
                        first_commit_message: String::from("Initial commit"),
                        path: String::from("~/src/github.com/himanoa/mkrepo")
                    }
                ]
            );
        }

        #[test]
        pub fn build_commands_return_to_create_directory_and_initialize_git_when_first_commit_message_is_none(
        ) {
            let c = Config {
                service: "github.com".to_string(),
                name: Some("himanoa".to_string()),
                ghq_root: Some("~/src".to_string()),
            };
            assert_eq!(
                build_commands(c, None, None, "mkrepo", None).unwrap(),
                vec![
                    CommandType::CreateDirectory {
                        path: String::from("~/src/github.com/himanoa/mkrepo")
                    },
                    CommandType::InitializeGit {
                        first_commit_message: String::from("Initial commit"),
                        path: String::from("~/src/github.com/himanoa/mkrepo")
                    }
                ]
            );
        }

        #[test]
        pub fn build_commands_return_to_create_directory_and_initialize_git_when_author_is_exist() {
            let c = Config {
                service: "github.com".to_string(),
                name: Some("himanoa".to_string()),
                ghq_root: Some("~/src".to_string()),
            };
            assert_eq!(
                build_commands(c, Some("h1manoa"), None, "mkrepo", None).unwrap(),
                vec![
                    CommandType::CreateDirectory {
                        path: String::from("~/src/github.com/h1manoa/mkrepo")
                    },
                    CommandType::InitializeGit {
                        first_commit_message: String::from("Initial commit"),
                        path: String::from("~/src/github.com/h1manoa/mkrepo")
                    }
                ]
            );
        }
    }
    #[test]
    pub fn build_commands_return_to_create_directory_and_initialize_git_when_service_is_exist() {
        let c = Config {
            service: "github.com".to_string(),
            name: Some("himanoa".to_string()),
            ghq_root: Some("~/src".to_string()),
        };
        assert_eq!(
            build_commands(c, None, Some("bitbucket.com"), "mkrepo", None).unwrap(),
            vec![
                CommandType::CreateDirectory {
                    path: String::from("~/src/bitbucket.com/himanoa/mkrepo")
                },
                CommandType::InitializeGit {
                    first_commit_message: String::from("Initial commit"),
                    path: String::from("~/src/bitbucket.com/himanoa/mkrepo")
                }
            ]
        );
    }
}
