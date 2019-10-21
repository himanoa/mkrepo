extern crate failure;

pub mod makerepo {
    use failure::Error;
    use std::fs::create_dir_all;
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

    pub fn create_directory(path: &str) -> Result<(), Error> {
        create_dir_all(path)?;
        Ok(())
    }

    pub fn initialize_git(first_commit_message: &str, path: &str) -> Result<(), Error> {
        Ok(())
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
}
