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

use std::{env, thread::sleep, time::Duration};

use anyhow::Result;
use clap::Parser;
use colored::{self, Colorize};
use env_logger;
use keyring::Entry;
use log::{self, LevelFilter, trace, warn};
use tokio;

use crate::{
    cli::{Cli, Commands, ConfigCommands},
    config::ArtemisConfig,
    core::{adapter::Adapter, git::ArtemisRepo},
};
mod cli;
mod config;
mod core;

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

async fn run_commands(cli: &Cli, cfg: &mut ArtemisConfig) -> Result<()> {
    match cli.command.as_ref().unwrap() {
        Commands::ListCourses => {
            let mut s = Adapter::init(30, cfg.get_base_url()).await.unwrap();

            let courses = s.get_all_courses().await.unwrap();
            for course in courses {
                println!("{:<5} {}", course.id, course.title)
            }
        }
        Commands::ListTasks { courseid } => {
            let mut s = Adapter::init(30, cfg.get_base_url()).await.unwrap();

            let courses = s.get_all_courses().await.unwrap();
            for course in courses {
                if course.id == *courseid {
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
        Commands::StartTask { taskid } => {
            let mut s = Adapter::init(30, cfg.get_base_url())
                .await
                .expect("adapter could not be started");
            let ssh_uri = s
                .srart_artemis_task(*taskid)
                .await
                .expect("couldnt start the task and fetch url");
            let repo =
                ArtemisRepo::create(&ssh_uri, *taskid).expect("couldn't create the repository");
            repo.commit_and_push()
                .expect("can't commit and push to remote repository");
        }
        Commands::Submit { taskid } => {
            let repo = ArtemisRepo::open(env::current_dir()?)?;
            repo.commit_and_push()?;
            sleep(Duration::from_secs(7));
            let mut s = Adapter::init(30, cfg.get_base_url()).await?;
            let test_results = s.get_latest_test_result(*taskid).await?; // TODO: make it so we get
            // taskid from the local repository, no need for it to be speciefied

            for test_result in test_results {
                println!(
                    "{:<4} {:<60} {}",
                    if test_result.passed {
                        "P".bold().green()
                    } else {
                        "F".bold().red()
                    },
                    test_result.name,
                    test_result.explanation.unwrap_or("".to_string()).red(),
                )
            }
        }
        Commands::Config { command } => match command {
            ConfigCommands::BaseUrl { url } => {
                cfg.set_base_url(url.clone());
                cfg.save(cli.cfg.as_deref());
            }
            ConfigCommands::Username { name } => {
                let uname =
                    Entry::new("artemiscli", "username").expect("can't create Entry for username");
                uname
                    .set_password(&name)
                    .expect("can't create Entry for password");
            }
            ConfigCommands::Password { password } => {
                let pwd =
                    Entry::new("artemiscli", "password").expect("can't create Entry for password");
                pwd.set_password(&password)?;
            }
        },
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let cli: Cli = Cli::parse();
    init_log(cli.verbosity);

    let mut config = ArtemisConfig::load(cli.cfg.as_deref());

    trace!("setup logging...");

    if cli.command.is_none() {
        warn!("command is none");
        return;
    }
    run_commands(&cli, &mut config).await.unwrap();
}
