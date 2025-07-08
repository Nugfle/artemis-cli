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

use clap::Parser;
use env_logger;
use keyring::Entry;
use log::{self, LevelFilter, trace, warn};
use tokio;

use crate::cli::{Cli, Commands};
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

#[tokio::main]
async fn main() {
    let cli: Cli = Cli::parse();
    init_log(cli.verbosity);

    trace!("setup logging...");

    if cli.command.is_none() {
        warn!("command is none");
        return;
    }
    trace!("parsing command...");
    match cli.command.unwrap() {
        Commands::ListCourses => {
            let mut s = core::scraper::Scraper::init(30).await.unwrap();

            let courses = s.get_all_courses().await.unwrap();
            for course in courses {
                println!("{:<5} {}", course.id, course.title)
            }
        }
        Commands::ListTasks { courseid } => {
            let mut s = core::scraper::Scraper::init(30).await.unwrap();

            let courses = s.get_all_courses().await.unwrap();
            for course in courses {
                if course.id == courseid {
                    for task in course.tasks {
                        println!(
                            "{:<5} {:<40} {:<15} {:<15}",
                            task.id,
                            task.title,
                            if task.is_active {
                                "active"
                            } else {
                                "not started"
                            },
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
        Commands::Submit => {}
        Commands::Config { username, password } => {
            if username.is_some() {
                let uname = Entry::new("artemiscli", "username").unwrap();
                uname.set_password(&username.unwrap()).unwrap();
            }
            if password.is_some() {
                let pwd = Entry::new("artemiscli", "password").unwrap();
                pwd.set_password(&password.unwrap()).unwrap();
            }
        }
    }
}
