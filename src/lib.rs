pub mod makerepo {
    use failure::{Error, Fail};
    use gitconfig::Value;
    use serde_derive::Deserialize;
    use std::fs::create_dir_all;
    use std::ops::Deref;
    use std::path::{Path, PathBuf};
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
        fn execute(&self, commands: Vec<CommandType>) -> Result<(), ExecutorError>;
    }

    #[derive(Debug, Default)]
    pub struct DryRunExecutor {}

    impl DryRunExecutor {
        pub fn new() -> DryRunExecutor {
            Self::default()
        }
    }

    impl Executor for DryRunExecutor {
        fn execute(&self, commands: Vec<CommandType>) -> Result<(), ExecutorError> {
            for command in commands {
                match command {
                    CommandType::CreateDirectory { path } => println!("CreateDirectory: {}", path),
                    CommandType::InitializeGit {
                        first_commit_message,
                        path,
                    } => println!("InitializeGit: {} {}", first_commit_message, path),
                }
            }
            Ok(())
        }
    }

    pub fn create_directory(path: &str) -> Result<(), std::io::Error> {
        create_dir_all(path)
    }

    #[derive(Debug, Fail)]
    pub enum GitError {
        #[fail(display = "fail git repository initialize")]
        Initialize,
        #[fail(display = "git repository already exist")]
        AlreadyExist,
    }

    impl From<std::io::Error> for GitError {
        fn from(_: std::io::Error) -> GitError {
            GitError::Initialize {}
        }
    }

    pub fn initialize_git(first_commit_message: &str, path: &str) -> Result<(), GitError> {
        let git_status_result = Command::new("git")
            .arg("status")
            .current_dir(path)
            .output()?;
        if git_status_result.status.success() {
            return Err(GitError::AlreadyExist {});
        }
        let create_dir_result = Command::new("git").arg("init").current_dir(path).output()?;
        let initial_commit_result = Command::new("git")
            .args(&["commit", "-m", first_commit_message, "--allow-empty"])
            .current_dir(path)
            .output()?;
        match (
            create_dir_result.status.success(),
            initial_commit_result.status.success(),
        ) {
            (true, true) => Ok(()),
            _ => Err(GitError::Initialize {}),
        }
    }

    #[derive(Debug, Default)]
    pub struct DefaultExecutor {}

    impl DefaultExecutor {
        pub fn new() -> DefaultExecutor {
            Self::default()
        }
    }

    #[derive(Debug, Fail)]
    pub enum ExecutorError {
        #[fail(display = "fail create directory")]
        CreateDirectroyError,
        #[fail(display = "fail initialize git repository")]
        GitInitializeError,
    }
    impl Executor for DefaultExecutor {
        fn execute(&self, commands: Vec<CommandType>) -> Result<(), ExecutorError> {
            let error = commands.into_iter().find_map(|command| {
                let result = match command {
                    CommandType::CreateDirectory { path } => {
                        create_directory(&path).map_err(|_| ExecutorError::CreateDirectroyError {})
                    }
                    CommandType::InitializeGit {
                        first_commit_message,
                        path,
                    } => initialize_git(&first_commit_message, &path)
                        .map_err(|_| ExecutorError::GitInitializeError {}),
                };
                if result.is_err() {
                    Some(result)
                } else {
                    None
                }
            });
            match error {
                None => Ok(()),
                Some(e) => match e {
                    Err(e) => Err(e),
                    _ => Ok(()),
                },
            }
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct Config {
        user_name: Option<String>,
        ghq_root: Option<PathBuf>,
        mkrepo_service: String,
        mkrepo_username: Option<String>,
    }

    #[derive(Debug, Fail)]
    pub enum FailLoadGitConfigError {
        #[fail(display = "fail git config --list --null stdout parse")]
        ParseError,
        #[fail(display = "fail git command execute error")]
        FailGitCommandExecuteError,
        #[fail(display = "Not found default service setting")]
        NotFoundDefaultServiceSetting,
    }
    pub fn load_git_config() -> Result<Config, FailLoadGitConfigError> {
        let command = Command::new("git")
            .args(&["config", "--list", "--null"])
            .output()
            .unwrap();
        let output = std::str::from_utf8(&command.stdout)
            .map_err(|_| FailLoadGitConfigError::FailGitCommandExecuteError)?;
        if let Some(git_config_map) = output.to_string().parse::<Value>().unwrap().as_object() {
            match git_config_map.get("mkrepo") {
                Some(Value::Object(mkrepo)) => match mkrepo.get("service") {
                    Some(Value::String(service)) => Ok(Config {
                        user_name: match git_config_map.get("user") {
                            Some(Value::Object(u)) => match u.get("name") {
                                Some(Value::String(name)) => Some(name.to_string()),
                                _ => None,
                            },
                            _ => None,
                        },
                        ghq_root: git_config_map
                            .get("ghq")
                            .and_then(Value::as_object)
                            .and_then(|ghq| ghq.get("root"))
                            .and_then(|root| match root {
                                Value::Object(_) => None,
                                Value::String(root) => Some(root),
                            })
                            .map(|root| {
                                if Path::new(root).starts_with("~") {
                                    let home = dirs::home_dir().unwrap_or_else(|| {
                                        unimplemented!("home directory not found")
                                    });
                                    Path::new(root).iter().skip(1).fold(home, |mut acc, comp| {
                                        acc.push(comp);
                                        acc
                                    })
                                } else if root.starts_with('~') {
                                    unimplemented!("(currently) unsupported use of \"~\"");
                                } else {
                                    root.into()
                                }
                            }),
                        mkrepo_service: service.to_string(),
                        mkrepo_username: mkrepo.get("username").and_then(
                            |username| match username {
                                Value::Object(_) => None,
                                Value::String(username) => Some(username.clone()),
                            },
                        ),
                    }),
                    Some(Value::Object(_)) => {
                        Err(FailLoadGitConfigError::NotFoundDefaultServiceSetting {})
                    }
                    None => {
                        println!("foobar");
                        Err(FailLoadGitConfigError::NotFoundDefaultServiceSetting {})
                    }
                },
                _ => Err(FailLoadGitConfigError::NotFoundDefaultServiceSetting {}),
            }
        } else {
            Err(FailLoadGitConfigError::ParseError {})
        }
    }

    pub fn build_commands<'a>(
        config: Config,
        author: Option<&'a str>,
        service_name: Option<&'a str>,
        repository_name: &'a str,
        first_commit_message: Option<&'a str>,
    ) -> Result<std::vec::Vec<CommandType>, Error> {
        let parent_path = config.ghq_root.as_ref().unwrap();
        let service = match service_name {
            Some(n) => n,
            None => config.mkrepo_service.as_ref(),
        };
        let repository_author = author
            .or_else(|| config.mkrepo_username.as_ref().map(Deref::deref))
            .or_else(|| config.user_name.as_ref().map(Deref::deref))
            .unwrap_or_else(|| {
                unimplemented!("`--author`, `mkrepo.username`, or `user.name` required")
            });
        let repository_path = parent_path
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
                user_name: Some("himanoa".to_owned()),
                ghq_root: Some("/home/user/src".into()),
                mkrepo_service: "github.com".to_owned(),
                mkrepo_username: None,
            };
            assert_eq!(
                build_commands(c, None, None, "mkrepo", Some("Initial commit")).unwrap(),
                vec![
                    CommandType::CreateDirectory {
                        path: String::from("/home/user/src/github.com/himanoa/mkrepo")
                    },
                    CommandType::InitializeGit {
                        first_commit_message: String::from("Initial commit"),
                        path: String::from("/home/user/src/github.com/himanoa/mkrepo")
                    }
                ]
            );
        }

        #[test]
        pub fn build_commands_return_to_create_directory_and_initialize_git_when_first_commit_message_is_none(
        ) {
            let c = Config {
                user_name: Some("himanoa".to_owned()),
                ghq_root: Some("/home/user/src".into()),
                mkrepo_service: "github.com".to_owned(),
                mkrepo_username: None,
            };
            assert_eq!(
                build_commands(c, None, None, "mkrepo", None).unwrap(),
                vec![
                    CommandType::CreateDirectory {
                        path: String::from("/home/user/src/github.com/himanoa/mkrepo")
                    },
                    CommandType::InitializeGit {
                        first_commit_message: String::from("Initial commit"),
                        path: String::from("/home/user/src/github.com/himanoa/mkrepo")
                    }
                ]
            );
        }

        #[test]
        pub fn build_commands_return_to_create_directory_and_initialize_git_when_author_is_exist() {
            let c = Config {
                user_name: Some("himanoa".to_owned()),
                ghq_root: Some("/home/user/src".into()),
                mkrepo_service: "github.com".to_owned(),
                mkrepo_username: None,
            };
            assert_eq!(
                build_commands(c, Some("h1manoa"), None, "mkrepo", None).unwrap(),
                vec![
                    CommandType::CreateDirectory {
                        path: String::from("/home/user/src/github.com/h1manoa/mkrepo")
                    },
                    CommandType::InitializeGit {
                        first_commit_message: String::from("Initial commit"),
                        path: String::from("/home/user/src/github.com/h1manoa/mkrepo")
                    }
                ]
            );
        }
    }
    #[test]
    pub fn build_commands_return_to_create_directory_and_initialize_git_when_service_is_exist() {
        let c = Config {
            user_name: Some("himanoa".to_owned()),
            ghq_root: Some("/home/user/src".into()),
            mkrepo_service: "github.com".to_owned(),
            mkrepo_username: None,
        };
        assert_eq!(
            build_commands(c, None, Some("bitbucket.com"), "mkrepo", None).unwrap(),
            vec![
                CommandType::CreateDirectory {
                    path: String::from("/home/user/src/bitbucket.com/himanoa/mkrepo")
                },
                CommandType::InitializeGit {
                    first_commit_message: String::from("Initial commit"),
                    path: String::from("/home/user/src/bitbucket.com/himanoa/mkrepo")
                }
            ]
        );
    }
}
