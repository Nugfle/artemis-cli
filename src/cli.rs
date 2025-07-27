/*
Copyright (C) 2025 Niklas Liesch <niklas.liesch@protonmail.com>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use clap::{Parser, Subcommand, command};
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(name = "artemiscli")]
#[command(about = "A CLI tool for intercating with artemis tasks")]
pub(crate) struct Cli {
    /// Verbosity of the output (use -v -vv -vvv to increase verbosity)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub(crate) verbosity: u8,

    #[arg(short, long)]
    pub(crate) cfg: Option<PathBuf>,

    #[command(subcommand)]
    pub(crate) command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum Commands {
    /// lists all enrolled courses on artemis
    ListCourses,
    /// lists all available tasks on artemis
    ListTasks {
        /// the id of the course as shown by list-courses
        courseid: u64,
    },
    /// start artemis task and clone the gl repository
    StartTask {
        /// the id of the task as given by list-task
        taskid: u64,
    },
    /// creates a commit, pushes to the repo and returns the test results
    Submit,
    /// fetches and prints the test results
    Fetch {
        /// the id of the task as given by list-task
        taskid: u64,
    },
    /// sets the global configuration for login data
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum ConfigCommands {
    Username { name: String },
    Password { password: String },
    BaseUrl { url: String },
}
