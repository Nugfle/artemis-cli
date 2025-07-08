/*
ArtemisCLI - a CLI tool to help students work with Artemis.

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

use std::env;

use anyhow::Result;
use clap::Parser;
use colored::{self, Colorize};
use env_logger;
use keyring::Entry;
use log::{self, LevelFilter, trace, warn};
use tokio;

use crate::{
    cli::{Cli, Commands},
    core::{adapter::Adapter, git::ArtemisRepo},
};
pub mod cli;
pub mod core;

fn init_log(verbosity: u8) {
    let log_level = match verbosity {
        0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    env_logger::builder()
        .filter_level(log_level)
        .target(env_logger::Target::Stdout)
        .init();
}

async fn run_commands(cli: Cli) -> Result<()> {
    match cli.command.unwrap() {
        Commands::ListCourses => {
            let mut s = Adapter::init(30).await.unwrap();

            let courses = s.get_all_courses().await.unwrap();
            for course in courses {
                println!("{:<5} {}", course.id, course.title)
            }
        }
        Commands::ListTasks { courseid } => {
            let mut s = Adapter::init(30).await.unwrap();

            let courses = s.get_all_courses().await.unwrap();
            for course in courses {
                if course.id == courseid {
                    for task in course.tasks {
                        println!(
                            "{:<5} {:<40} {:<15}",
                            task.id,
                            task.title,
                            if task.completed {
                                "completed"
                            } else if task.is_active {
                                "incomplete"
                            } else {
                                "not started"
                            }
                        )
                    }
                }
            }
        }
        Commands::StartTask { taskid } => {}
        Commands::Submit => {
            let repo = ArtemisRepo::open(env::current_dir()?)?;
            repo.commit_and_push();

            let mut s = Adapter::init(30).await?;
            let taskid = 220; // TODO: this should be in read from a config file in the project dir
            let test_results = s.get_latest_test_result(taskid).await?;

            for test_result in test_results {
                println!(
                    "{} {} {}",
                    if test_result.passed {
                        "P".bold().green()
                    } else {
                        "F".bold().red()
                    },
                    test_result.name,
                    test_result.explanation.unwrap_or("".to_string()),
                )
            }
        }
        Commands::Config { username, password } => {
            if username.is_some() {
                let uname =
                    Entry::new("artemiscli", "username").expect("can't create Entry for username");
                uname
                    .set_password(&username.unwrap())
                    .expect("can't create Entry for password");
            }
            if password.is_some() {
                let pwd = Entry::new("artemiscli", "password")?;
                pwd.set_password(&password.unwrap())?;
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let cli: Cli = Cli::parse();
    init_log(cli.verbosity);

    trace!("setup logging...");

    if cli.command.is_none() {
        warn!("command is none");
        return;
    }
    run_commands(cli).await.unwrap();
}
