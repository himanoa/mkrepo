pub mod makerepo {
    use std::fs::create_dir_all;
    #[derive(Debug)]
    pub enum CommandType<'a> {
        CreateDirectory {
            repository: &'a str,
            author: &'a str,
            hosting_service: &'a str,
        },
        InitializeGit {
            first_commit_message: &'a str,
        },
    }

    pub trait Executor {
        fn execute(&self, commands: Vec<CommandType>) -> Result<(), ()>;
    }

    pub struct DryRunExecutor {}

    impl DryRunExecutor {
        pub fn new() -> DryRunExecutor {
            DryRunExecutor {}
        }
    }

    impl Executor for DryRunExecutor {
        fn execute(&self, commands: Vec<CommandType>) -> Result<(), ()> {
            for command in commands {
                match command {
                    CommandType::CreateDirectory {
                        repository,
                        author,
                        hosting_service,
                    } => println!("CreateDirectory: {}, {}, {}", repository, author, hosting_service),
                    CommandType::InitializeGit {
                        first_commit_message,
                    } => println!("InitializeGit {}", first_commit_message),
                }
            }
            Ok(())
        }
    }

    pub fn create_directory(path: &str) -> Result<(), ()> {
        create_dir_all(path)?;
        Ok(())
    }
}
