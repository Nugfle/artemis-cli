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
use std::{
    fmt::{Display, write},
    sync::Arc,
    time::Duration,
};

use anyhow::{Result, anyhow};
use chrono::{DateTime, FixedOffset};
use colored::Colorize;
use keyring::Entry;
use log::{debug, error, info, trace};
use reqwest::{
    Client, Response,
    cookie::{CookieStore, Jar},
};
use serde::Deserialize;
use serde_json::{Value, json};

pub struct Adapter {
    client: Client,
    cookies: Arc<Jar>,
}

#[derive(Clone, Debug, Default)]
pub struct Task {
    pub(crate) title: String,
    pub(crate) id: u64,
    pub(crate) is_active: bool,
    pub(crate) completed: bool,
    pub(crate) repo_uri: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Course {
    pub(crate) id: u64,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) tasks: Vec<Task>,
}

#[derive(Clone, Debug)]
pub struct Test {
    pub(crate) name: String,
    pub(crate) passed: bool,
    pub(crate) explanation: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LogStatement {
    pub(crate) id: u64,
    pub(crate) time: DateTime<FixedOffset>,
    pub(crate) log: String,
}

impl Display for LogStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write(
            f,
            format_args!(
                "{:<30} {}",
                self.time,
                if self.log[0..7] == *"[ERROR]" {
                    self.log.red()
                } else if self.log[0..6] == *"[INFO]" {
                    self.log.bright_blue()
                } else {
                    self.log.normal()
                }
            ),
        )
    }
}

impl Adapter {
    pub async fn init(timeout: u8) -> Result<Self> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            "Mozilla/5.0 (compatible; RustScraper/1.0)".parse().unwrap(),
        );
        headers.insert(
            reqwest::header::ACCEPT,
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
                .parse()
                .unwrap(),
        );

        let jar = Arc::new(Jar::default());
        let entry =
            Entry::new("artemiscli", "jwt-token").expect("cant create keyring entry for jwt token");
        let mut restored_cookie = false;
        if let Ok(cookie) = entry.get_password() {
            jar.add_cookie_str(
                &cookie,
                &reqwest::Url::parse("https://artemis-app.inf.tu-dresden.de").unwrap(),
            );
            restored_cookie = true;
        }

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(timeout as u64))
            .cookie_store(true)
            .cookie_provider(jar.clone())
            .build()
            .expect("cant build the reqwest client");

        let mut s = Self {
            client,
            cookies: jar,
        };

        if !restored_cookie {
            s.login().await?;
        }

        Ok(s)
    }

    pub async fn login(&mut self) -> Result<()> {
        let uname =
            Entry::new("artemiscli", "username").expect("cant create keyring entry for username");
        let pwd =
            Entry::new("artemiscli", "password").expect("cant create keyring entry for password");

        let auth = json!({
            "username": uname.get_password().expect("you havent configured a username yet, use 'artemis-cli config [USERNAME] [PASSWORD]' and try again"),
            "password": pwd.get_password().expect("you havent configured a password yet, use 'artemis-cli config [USERNAME] [PASSWORD]' and try again"),
            "rememberMe": true,
        });
        let response = self
            .client
            .post("https://artemis-app.inf.tu-dresden.de/api/public/authenticate")
            .json(&auth)
            .send()
            .await?;

        if response.status().is_success() {
            info!("succesfully logged in");
            let entry = Entry::new("artemiscli", "jwt-token")?;
            entry
                .set_password(
                    self.cookies
                        .cookies(&reqwest::Url::parse(
                            "https://artemis-app.inf.tu-dresden.de",
                        )?)
                        .unwrap()
                        .to_str()?,
                )
                .unwrap();
            Ok(())
        } else {
            error!("cant log in to artemis {:?}", response.status());
            Err(anyhow!("login failed, aborting..."))
        }
    }

    fn parse_task(raw_task: &Value) -> Result<Task> {
        let task_id = raw_task.get("id").unwrap().as_u64().unwrap();
        let task_title = raw_task.get("title").unwrap().as_str().unwrap().to_string();
        let active = raw_task.get("studentParticipations");

        if active.is_none() {
            let task = Task {
                is_active: false,
                completed: false,
                id: task_id,
                title: task_title,
                repo_uri: None,
            };
            return Ok(task);
        }

        let participation_info = raw_task
            .get("studentParticipations")
            .unwrap()
            .as_array()
            .unwrap()
            .first()
            .unwrap();

        let repo_uri = participation_info
            .get("repositoryUri")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();

        if participation_info.get("results").is_none() {
            let task = Task {
                title: task_title,
                id: task_id,
                completed: false,
                repo_uri: Some(repo_uri),
                is_active: true,
            };
            return Ok(task);
        }

        let completed = participation_info
            .get("results")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .fold(false, |acc, e| {
                acc | (e.get("score").unwrap().as_f64().unwrap() == 100.0)
            });

        let task = Task {
            title: task_title,
            id: task_id,
            completed,
            repo_uri: Some(repo_uri),
            is_active: true,
        };
        return Ok(task);
    }

    fn parse_course(course: &Value) -> Result<Course> {
        trace!("parsing course ... ");
        let course_title = course.get("title").unwrap().as_str().unwrap().to_string();

        let course_id = course.get("id").unwrap().as_u64().unwrap();

        let raw_tasks = course.get("exercises").unwrap().as_array().unwrap();
        let mut tasks = Vec::new();

        trace!("fetching {} tasks...", raw_tasks.len());
        for raw_task in raw_tasks {
            tasks.push(Self::parse_task(raw_task).unwrap());
        }

        let description = course
            .get("description")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();

        Ok(Course {
            description,
            id: course_id,
            title: course_title,
            tasks,
        })
    }

    async fn fetch_json(&mut self, uri: &str) -> Result<Response> {
        let response = self
            .client
            .get(uri)
            .header("Accept", "application/json")
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            self.login().await?;
        }
        if !response.status().is_success() {
            error!("coudn't fetch json from {}", uri);
            return Err(anyhow!("coudn't fetch json from {}", uri));
        }
        Ok(response)
    }

    pub async fn get_all_courses(&mut self) -> Result<Vec<Course>> {
        debug!("fetching course names...");

        let text = self
            .fetch_json("https://artemis-app.inf.tu-dresden.de/api/courses/for-dashboard")
            .await?
            .text()
            .await?;

        let mut deserializer = serde_json::Deserializer::from_str(&text);
        let json = Value::deserialize(&mut deserializer)?;

        trace!("start deserializing courses page...");
        let courses = json.get("courses").unwrap();
        let raw_course_array = courses.as_array().unwrap();

        let mut course_list = Vec::new();

        for course_info in raw_course_array {
            let course = course_info.get("course").unwrap();
            course_list.push(Self::parse_course(course).unwrap());
        }

        Ok(course_list)
    }

    fn parse_exercise_details(text: &str) -> Result<(u64, u64, bool)> {
        let mut deserializer = serde_json::Deserializer::from_str(text);
        let json = Value::deserialize(&mut deserializer)?;
        let exercise = json.get("exercise").unwrap();
        let participation = exercise
            .get("studentParticipations")
            .unwrap()
            .as_array()
            .unwrap()
            .first()
            .unwrap();

        let participation_id = participation.get("id").unwrap().as_u64().unwrap();
        let results = participation
            .get("results")
            .expect("there are no results available yet")
            .as_array()
            .unwrap();

        let mut submissions = Vec::new();
        for result in results {
            let result_id = result.get("id").unwrap().as_u64().unwrap();
            let completion_time = result.get("completionDate").unwrap().as_str().unwrap();
            let timestamp = DateTime::parse_from_rfc3339(completion_time).unwrap();

            let build_failiure = result
                .get("submission")
                .unwrap()
                .get("buildFailed")
                .unwrap()
                .as_bool()
                .unwrap();

            submissions.push((timestamp, result_id, build_failiure));
        }
        let (_, resutl_id, build_faliure) = submissions
            .iter()
            .max_by(|(ts1, _, _), (ts2, _, _)| ts1.cmp(ts2))
            .unwrap();

        Ok((participation_id, *resutl_id, *build_faliure))
    }

    fn parse_test_result_details(text: String) -> Result<Vec<Test>> {
        let mut deserializer = serde_json::Deserializer::from_str(&text);
        let json = Value::deserialize(&mut deserializer)?;
        let raw_tests = json.as_array().unwrap();

        let mut tests = Vec::new();

        for raw_test in raw_tests {
            let passed = raw_test.get("positive").unwrap().as_bool().unwrap();
            let name = raw_test
                .get("testCase")
                .unwrap()
                .get("testName")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string();
            let explanation = if !passed {
                Some(
                    raw_test
                        .get("detailText")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string(),
                )
            } else {
                None
            };
            let test = Test {
                name,
                passed,
                explanation,
            };
            tests.push(test);
        }

        Ok(tests)
    }

    pub async fn get_latest_test_result(&mut self, taskid: u64) -> Result<Vec<Test>> {
        let details_uri = format!(
            "https://artemis-app.inf.tu-dresden.de/api/exercises/{}/details",
            taskid
        );
        let text = self.fetch_json(&details_uri).await?.text().await?;

        let (participation_id, result_id, build_failiure) =
            Self::parse_exercise_details(&text).unwrap();

        if build_failiure {
            let buildlogs_url = format!(
                "https://artemis-app.inf.tu-dresden.de/api/repository/{}/buildlogs?resultId={}",
                participation_id, result_id
            );

            let buildlogs: Vec<LogStatement> =
                self.fetch_json(&buildlogs_url).await?.json().await?;

            println!("{}", "BUILD FAILIURE:".red().bold());
            for log in buildlogs {
                println!("{}", log);
            }

            return Ok(Vec::new());
        }

        let test_result_uri = format!(
            "https://artemis-app.inf.tu-dresden.de/api/participations/{}/results/{}/details",
            participation_id, result_id,
        );

        let test_result_text = self.fetch_json(&test_result_uri).await?.text().await?;

        Self::parse_test_result_details(test_result_text.to_owned())
    }
}
