use std::path::PathBuf;

use clap::{Parser, Subcommand, command};

#[derive(Parser, Debug, Clone)]
#[command(name = "artemiscli")]
#[command(about = "A CLI tool for intercating with artemis tasks")]
pub(crate) struct Cli {
    /// Verbosity of the output (use -v -vv -vvv to increase verbosity)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub(crate) verbosity: u8,

    #[command(subcommand)]
    pub(crate) command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum Commands {
    /// lists all enrolled courses on artemis
    ListCourses,
    /// lists all available tasks on artemis
    ListTasks {
        /// the id of the course as shown by ListCourses
        courseid: u64,
    },
    /// start artemis task and clone the gl repository
    StartTask {
        /// the id of the task as given by listtask
        taskid: u64,
    },
    /// creates a commit, pushes to the repo and returns the test results
    Submit,
    /// sets the global configuration for login data
    Config {
        /// your artemis username
        username: Option<String>,
        /// your artemis password
        password: Option<String>,
    },
}
