use anyhow::Result;
use git2::{Cred, PushOptions, RemoteCallbacks, Repository, Signature};
use log::{info, trace};
use std::{env, path::Path};

pub struct ArtemisRepo {
    repo: Repository,
}

impl ArtemisRepo {
    pub fn create(url: &str) -> Result<Self> {
        let repo = Repository::clone(url, env::current_dir()?)?;
        Ok(Self { repo })
    }
    pub fn open<T>(path: T) -> Result<Self>
    where
        T: AsRef<Path>,
    {
        let repo = Repository::open(path)?;
        Ok(Self { repo })
    }

    pub fn commit_and_push(&self) -> Result<()> {
        let mut index = self.repo.index()?;

        trace!("indexing files...");
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        trace!("creating tree...");
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;

        trace!("selecting partent...");
        let head = self.repo.head()?;
        let parent = head.peel_to_commit()?;

        trace!("loading name and email from config...");
        let config = git2::Config::open_default()?;
        let name = config
            .get_string("user.name")
            .expect("no username for git configured. Run git config --global user.name 'YourName'");
        let email = config
            .get_string("user.email")
            .expect("no email for git configured. Run git config --global user.email 'YourEmail'");

        let signature = Signature::now(&name, &email)?;

        trace!("running commit...");
        let commit_id = self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "automated commit...",
            &tree,
            &[&parent],
        )?;
        info!("successfully commited {}", commit_id);

        self.push()?;

        Ok(())
    }

    pub fn push(&self) -> Result<()> {
        trace!("trying to find remote...");
        let mut remote = self.repo.find_remote("origin")?;

        let mut callbacks = RemoteCallbacks::new();
        trace!("adding callback...");
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        });
        callbacks.transfer_progress(|progress| {
            info!("Progress: {} Bytes", progress.received_bytes());
            true
        });

        // Configure push options
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        trace!("pushing...");
        remote.push(
            &["refs/heads/main:refs/heads/main"],
            Some(&mut push_options),
        )?;

        info!("successfully pushed to remote");

        Ok(())
    }
}
