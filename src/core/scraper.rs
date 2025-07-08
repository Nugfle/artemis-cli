use std::{sync::Arc, time::Duration};

use anyhow::{Result, anyhow};
use keyring::Entry;
use log::{debug, error, info, trace};
use reqwest::{
    Client,
    cookie::{CookieStore, Jar},
};
use serde::Deserialize;
use serde_json::{Value, json};

pub struct Scraper {
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

impl Scraper {
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
            let uname = Entry::new("artemiscli", "username")
                .expect("cant create keyring entry for username");
            let pwd = Entry::new("artemiscli", "password")
                .expect("cant create keyring entry for password");
            s.login(
                uname.get_password().expect("you havent configured a username yet, use 'artemis-cli config [USERNAME] [PASSWORD]' and try again"),
                pwd.get_password().expect("you havent configured a password yet, use 'artemis-cli config [USERNAME] [PASSWORD]' and try again")
            ).await?;
        }

        Ok(s)
    }

    pub async fn login(&mut self, username: String, password: String) -> Result<()> {
        let auth = json!({
            "username": username,
            "password": password,
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

    pub async fn get_all_courses(&mut self) -> Result<Vec<Course>> {
        debug!("fetching course names...");
        let response = self
            .client
            .get("https://artemis-app.inf.tu-dresden.de/api/courses/for-dashboard")
            .header("Accept", "application/json")
            .send()
            .await?;

        let text = response.text().await?;

        let mut deserializer = serde_json::Deserializer::from_str(&text);
        let json = Value::deserialize(&mut deserializer)?;

        trace!("start deserializing...");
        let courses = json.get("courses").unwrap();
        let raw_course_array = courses.as_array().unwrap();

        let mut course_list = Vec::new();

        for course_info in raw_course_array {
            let course = course_info.get("course").unwrap();

            let course_title = course.get("title").unwrap().as_str().unwrap().to_string();
            trace!("got course title {:?}", course_title);

            let course_id = course.get("id").unwrap().as_u64().unwrap();
            trace!("got course id {:?}", course_id);

            let raw_tasks = course.get("exercises").unwrap().as_array().unwrap();
            let mut tasks = Vec::new();

            trace!("fetching {} tasks", raw_tasks.len());
            for raw_task in raw_tasks {
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
                    tasks.push(task);
                    continue;
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
                    tasks.push(task);
                    continue;
                }

                let completed = participation_info
                    .get("results")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .first()
                    .unwrap()
                    .get("score")
                    .unwrap()
                    .as_f64()
                    .unwrap()
                    == 100.0;

                let task = Task {
                    title: task_title,
                    id: task_id,
                    completed,
                    repo_uri: Some(repo_uri),
                    is_active: true,
                };
                tasks.push(task);
            }

            let description = course
                .get("description")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string();
            trace!("got course description {:?}", description);

            course_list.push(Course {
                description,
                id: course_id,
                title: course_title,
                tasks,
            });
        }

        Ok(course_list)
    }
}
