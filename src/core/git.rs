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

use anyhow::Result;
use git2::{
    Cred, CredentialType, FetchOptions, PushOptions, RemoteCallbacks, Repository, Signature,
    build::RepoBuilder,
};
use log::{error, info, trace};
use std::{env, path::Path};

pub struct ArtemisRepo {
    repo: Repository,
}

impl ArtemisRepo {
    pub fn create(url: &str) -> Result<Self> {
        let mut path = env::current_dir().expect("can't access current directory");
        path.push("artemis-task");

        // Remove existing directory if it exists
        if path.exists() {
            std::fs::remove_dir_all(&path).unwrap();
        }

        info!("start cloning: {} into {}...", url, path.to_str().unwrap());

        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|url, username_from_url, allowed_types| {
            info!("Credential callback called for URL: {}", url);
            info!("Username from URL: {:?}", username_from_url);
            info!("Allowed types: {:?}", allowed_types);

            if allowed_types.contains(CredentialType::SSH_KEY) {
                let username = username_from_url.unwrap_or("git");
                info!("Trying SSH key authentication for user: {}", username);
                Cred::ssh_key_from_agent(username)
            } else if allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT) {
                // For HTTPS URLs, you might need username/password or token
                error!("Password authentication not implemented");
                Err(git2::Error::from_str(
                    "password authentication not implemented",
                ))
            } else {
                error!(
                    "No supported credential type available: {:?}",
                    allowed_types
                );
                Err(git2::Error::from_str("no supported credential type"))
            }
        });

        let mut fo = FetchOptions::new();
        fo.remote_callbacks(callbacks);

        let mut builder = RepoBuilder::new();
        builder.fetch_options(fo);
        builder.branch("main");

        let repo = builder.clone(url, &path).map_err(|e| {
            error!("Failed to clone repository: {}", e);
            // Print more detailed error information
            match e.class() {
                git2::ErrorClass::Net => error!("Network error - check your connection and URL"),
                git2::ErrorClass::Ssh => {
                    error!("SSH error - check your SSH key and authentication")
                }
                git2::ErrorClass::Http => error!("HTTP error - check your credentials and URL"),
                _ => error!("Other git error: {:?}", e.class()),
            }
            e
        })?;

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
        self.commit()?;
        self.push()?;
        Ok(())
    }

    pub fn commit(&self) -> Result<()> {
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
