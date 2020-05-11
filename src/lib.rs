pub mod makerepo {
    use failure::{Error, Fail};
    use git2::Config as GitConfig;
    use git2::ConfigEntries;
    use serde_derive::Deserialize;
    use shellexpand::tilde;
    use std::fs::create_dir_all;
    use std::iter::Iterator;
    use std::ops::Deref;
    use std::path;
    use std::path::PathBuf;
    use std::fs::File;
    use flate2::read::GzDecoder;
    use tar::Archive;
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
        ExpandProjectTemplate {
            template_name: String,
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
                    CommandType::ExpandProjectTemplate { template_name, path } => println!("ExpandProjectTemplate: {} {}", template_name, path),
                }
            }
            Ok(())
        }
    }

    pub fn expand_project_template(path: &str, project_name: &str) -> Result<(), std::io::Error> {
        let archive_path = normalize_seps(format!("~/.mkrepo/{}.tar.gz", project_name).as_ref());

        let tar_gz = File::open(archive_path)?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        archive.unpack(path)?;
        Ok(())
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
        #[fail(display = "fail expand_project_template")]
        ExpandProjectTempalteError,
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
                    CommandType::ExpandProjectTemplate { path, template_name }  => expand_project_template(&path, &template_name).map_err(|_| ExecutorError::ExpandProjectTempalteError),
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
        #[fail(display = "fail load default git config")]
        LoadError,
        #[fail(display = "fail git config --list --null stdout parse")]
        ParseError,
        #[fail(display = "fail git command execute error")]
        FailGitCommandExecuteError,
        #[fail(display = "Not found default service setting")]
        NotFoundDefaultServiceSetting,
    }

    impl std::convert::From<git2::Error> for FailLoadGitConfigError {
        fn from(_: git2::Error) -> Self {
            Self::LoadError {}
        }
    }

    pub fn fetch_value(config: &ConfigEntries, key_name: &str) -> Option<String> {
        let mut matches = config.filter_map(|e| {
            if let Ok(entry) = e.as_ref() {
                match (&entry.name(), &entry.value()) {
                    (Some(n), Some(v)) => {
                        if n == &key_name {
                            Some(String::from(*v))
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            } else {
                None
            }
        });
        matches.next()
    }
    pub fn load_git_config(config: GitConfig) -> Result<Config, FailLoadGitConfigError> {
        if let Some(service) = fetch_value(&config.entries(None)?, "mkrepo.service") {
            Ok(Config {
                user_name: fetch_value(&config.entries(None)?, "user.name"),
                mkrepo_service: service,
                mkrepo_username: fetch_value(&config.entries(None)?, "mkrepo.username"),
                ghq_root: fetch_value(&config.entries(None)?, "ghq.root")
                    .map(|root| PathBuf::from(tilde(&root).as_ref())),
            })
        } else {
            Err(FailLoadGitConfigError::NotFoundDefaultServiceSetting)
        }
    }

    pub fn build_commands<'a>(
        config: Config,
        author: Option<&'a str>,
        service_name: Option<&'a str>,
        repository_name: &'a str,
        first_commit_message: Option<&'a str>,
        project_name: Option<&'a str>,
    ) -> Result<std::vec::Vec<CommandType>, Error> {
        let parent_path = config.ghq_root.as_ref().expect("ghq.root is not defined.");
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

        let mut commands = vec![
            CommandType::CreateDirectory {
                path: normalize_seps(repository_path.to_str().unwrap()),
            },
            CommandType::InitializeGit {
                path: normalize_seps(repository_path.to_str().unwrap()),
                first_commit_message: String::from(match first_commit_message {
                    Some(x) => x,
                    None => "Initial commit",
                }),
            },
        ];

        if project_name.is_some() {
            commands.push(CommandType::ExpandProjectTemplate { path: normalize_seps(repository_path.to_str().unwrap()), template_name: project_name.unwrap().to_string() })
        }
        Ok(commands)
    }

    fn normalize_seps(path: &str) -> String {
        path.replace(path::is_separator, &path::MAIN_SEPARATOR.to_string())
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        use std::fs::File;
        use tempfile::TempDir;

        #[test]
        pub fn build_commands_return_to_create_directory_and_initialize_git() {
            let c = Config {
                user_name: Some("himanoa".to_owned()),
                ghq_root: Some(normalize_seps("/home/user/src").into()),
                mkrepo_service: "github.com".to_owned(),
                mkrepo_username: None,
            };
            assert_eq!(
                build_commands(c, None, None, "mkrepo", Some("Initial commit"), None).unwrap(),
                vec![
                    CommandType::CreateDirectory {
                        path: normalize_seps("/home/user/src/github.com/himanoa/mkrepo")
                    },
                    CommandType::InitializeGit {
                        first_commit_message: String::from("Initial commit"),
                        path: normalize_seps("/home/user/src/github.com/himanoa/mkrepo")
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
                build_commands(c, None, None, "mkrepo", None, None).unwrap(),
                vec![
                    CommandType::CreateDirectory {
                        path: normalize_seps("/home/user/src/github.com/himanoa/mkrepo")
                    },
                    CommandType::InitializeGit {
                        first_commit_message: String::from("Initial commit"),
                        path: normalize_seps("/home/user/src/github.com/himanoa/mkrepo")
                    }
                ]
            );
        }

        #[test]
        pub fn build_commands_return_to_create_directory_and_initialize_git_when_author_is_exist() {
            let c = Config {
                user_name: Some("himanoa".to_owned()),
                ghq_root: Some(normalize_seps("/home/user/src").into()),
                mkrepo_service: "github.com".to_owned(),
                mkrepo_username: None,
            };
            assert_eq!(
                build_commands(c, Some("h1manoa"), None, "mkrepo", None, None).unwrap(),
                vec![
                    CommandType::CreateDirectory {
                        path: normalize_seps("/home/user/src/github.com/h1manoa/mkrepo")
                    },
                    CommandType::InitializeGit {
                        first_commit_message: String::from("Initial commit"),
                        path: normalize_seps("/home/user/src/github.com/h1manoa/mkrepo")
                    }
                ]
            );
        }
        #[test]
        pub fn build_commands_return_to_create_directory_and_initialize_git_when_service_is_exist()
        {
            let c = Config {
                user_name: Some("himanoa".to_owned()),
                ghq_root: Some(normalize_seps("/home/user/src").into()),
                mkrepo_service: "github.com".to_owned(),
                mkrepo_username: None,
            };
            assert_eq!(
                build_commands(c, None, Some("bitbucket.com"), "mkrepo", None, None).unwrap(),
                vec![
                    CommandType::CreateDirectory {
                        path: normalize_seps("/home/user/src/bitbucket.com/himanoa/mkrepo")
                    },
                    CommandType::InitializeGit {
                        first_commit_message: String::from("Initial commit"),
                        path: normalize_seps("/home/user/src/bitbucket.com/himanoa/mkrepo")
                    }
                ]
            );
        }

        #[test]
        pub fn build_commands_return_to_create_directory_and_initialize_git_when_service_and_expand_project_template_is_exist()
        {
            let c = Config {
                user_name: Some("himanoa".to_owned()),
                ghq_root: Some(normalize_seps("/home/user/src").into()),
                mkrepo_service: "github.com".to_owned(),
                mkrepo_username: None,
            };
            assert_eq!(
                build_commands(c, None, Some("bitbucket.com"), "mkrepo", None, Some("typescript")).unwrap(),
                vec![
                    CommandType::CreateDirectory {
                        path: normalize_seps("/home/user/src/bitbucket.com/himanoa/mkrepo")
                    },
                    CommandType::InitializeGit {
                        first_commit_message: String::from("Initial commit"),
                        path: normalize_seps("/home/user/src/bitbucket.com/himanoa/mkrepo")
                    },
                    CommandType::ExpandProjectTemplate {
                        template_name: String::from("typescript"),
                        path: normalize_seps("/home/user/src/bitbucket.com/himanoa/mkrepo")
                    }
                ]
            );
        }
        #[test]
        pub fn fetch_value_return_to_value() {
            let td = TempDir::new().unwrap();
            let path = td.path().join("foo");
            File::create(&path).unwrap();
            let mut c = GitConfig::open(&path).unwrap();
            assert!(c.get_str("a.foo").is_err());
            c.set_str("a.foo", "foobar1").unwrap();
            c.set_str("a.bar", "foobar2").unwrap();
            assert_eq!(
                fetch_value(&c.entries(None).unwrap(), "a.foo"),
                Some(String::from("foobar1"))
            );
            assert_eq!(
                fetch_value(&c.entries(None).unwrap(), "a.bar"),
                Some(String::from("foobar2"))
            );
        }
    }
}
