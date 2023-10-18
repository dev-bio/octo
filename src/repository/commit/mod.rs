
use std::{

    borrow::{Cow}, 
    path::{Path}, 

    fmt::{
    
        Formatter as FmtFormatter,
        Display as FmtDisplay,
        Result as FmtResult,
    }, 

    io::{Cursor}, 
};

use serde::{Deserialize};

pub mod compare;
pub use compare::{
    
    CompareError,
    CompareFile,
    Compare, 
};

use thiserror::{Error};
use zip::{ZipArchive};

use crate::{

    repository::{

        reference::{ReferenceError},
        
        tree::{Tree},
        sha::{Sha}, 

        HandleRepositoryError,
        HandleRepository,
    },

    common::{Date},

    client::{

        ClientError,
        Client, ClientResponseError, 
    },

    models::common::commit::{Commit},

    GitHubProperties,
    GitHubResult, 
};

#[derive(Error, Debug)]
pub enum CommitError {
    #[error("Compare error!")]
    Compare(#[from] CompareError),
    #[error("Reference error!")]
    Reference(#[from] ReferenceError),
    #[error("Client error!")]
    Client(#[from] ClientError),
    #[error("Commit not found: '{commit}'")]
    Nothing { commit: Sha<'static> },
}

#[derive(Clone, Debug)]
pub struct HandleCommit<'a> {
    pub(crate) repository: HandleRepository<'a>,
    pub(crate) date: Date,
    pub(crate) sha: Sha<'a>,
}

impl<'a> HandleCommit<'a> {
    pub(crate) fn try_fetch(repository: HandleRepository<'a>, commit: impl Into<Sha<'a>>) -> GitHubResult<HandleCommit<'a>, CommitError> {
        let commit = commit.into()
            .to_owned();

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct CapsuleAuthor {
            date: Date,
        }    

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            author: CapsuleAuthor,
            sha: Sha<'static>,
        }

        let Capsule { sha, author: CapsuleAuthor { date } } = {

            let request = {

                repository.get_client()
                    .get(format!("repos/{repository}/git/commits/{commit}"))?
            };

            match request.send() {
                Ok(response) => response.json()?,
                Err(error) => match error {
                    ClientError::Response(ClientResponseError::Nothing { .. }) => {
                        return Err(CommitError::Nothing { commit });
                    },
                    error => return Err(error.into()),
                }
            }
        };

        Ok(HandleCommit {
            repository,
            date,
            sha,
        })
    }

    pub(crate) fn try_create(repository: HandleRepository<'a>, parents: impl AsRef<[HandleCommit<'a>]>, tree: Tree, message: impl AsRef<str>) -> GitHubResult<HandleCommit<'a>, CommitError> {
        #[derive(Debug)]
        #[derive(Deserialize)]
        struct CapsuleAuthor {
            date: Date,
        }    

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            author: CapsuleAuthor,
            sha: Sha<'static>,
        }

        let Capsule { sha, author: CapsuleAuthor { date } } = {

            let parents = parents.as_ref();
            let message = message.as_ref();

            let parents: Vec<Sha> = parents.iter()
                .map(|commit| commit.get_sha())
                .collect();

            let ref payload = serde_json::json!({
                "parents": parents.as_slice(),
                "message": message.to_owned(),
                "tree": tree.get_sha(),
            });
            
            repository.get_client()
                .post(format!("repos/{repository}/git/commits"))?
                .json(payload)
                .send()?
                .json()?
        };

        Ok(HandleCommit {
            repository,
            date,
            sha,
        })
    }

    pub fn try_compare(&self, head: HandleCommit<'a>) -> GitHubResult<Compare, CommitError> {
        Ok(Compare::try_from_base_head(self.repository.clone(), self.clone(), head)?)
    }

    pub fn try_get_parents(&self) -> GitHubResult<Vec<HandleCommit>, CommitError> {
        let Self { repository, .. } = { self };

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct CapsuleParents {
            sha: Sha<'static>
        }

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            parents: Vec<CapsuleParents>
        }

        let Capsule { parents } = {

            repository.get_client()
                .get(format!("repos/{repository}/git/commits/{self}"))?
                .send()?
                .json()?
        };

        let mut collection = Vec::new();
        for CapsuleParents { sha } in parents.iter() {
            collection.push(HandleCommit::try_fetch(repository.clone(), {
                sha.clone()
            })?);
        }

        Ok(collection)
    }

    pub fn try_get_tree(&self, recursive: bool) -> GitHubResult<Tree, HandleRepositoryError> {
        let Self { repository, .. } = { self };

        let client = self.get_client();

        let response = client.get(format!("repos/{repository}/git/commits/{self}"))?
            .send()?;

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct CapsuleTree {
            sha: Sha<'static>
        }

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            tree: CapsuleTree
        }

        let Capsule { 
            tree: CapsuleTree { sha } 
        } = response.json()?;

        Ok(Tree::try_fetch(repository, sha, recursive)?)
    }

    pub fn try_get_date(&'a self) -> GitHubResult<Date, CommitError> {
        let repository = self.get_parent();

        let response = {

            repository.get_client()
                .get(format!("repos/{repository}/git/commits/{self}"))?
                .send()?
        };

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct CapsuleAuthor {
            date: Date
        }

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            author: CapsuleAuthor
        }

        let Capsule { 
            author: CapsuleAuthor { date } 
        } = response.json()?;

        Ok(date)
    }

    pub fn try_download(&self, path: impl AsRef<Path>) -> GitHubResult<(), HandleRepositoryError> {
        let Self { repository, .. } = { self };

        let cursor = Cursor::new({
            
            repository.get_client()
                .get(format!("repos/{repository}/zipball/{self}"))?
                .send()?
                .bytes()?
        });

        Ok(ZipArchive::new(cursor)?
            .extract(path.as_ref())?)
    }

    pub fn get_date(&self) -> Date {
        self.date.clone()
    }

    pub fn get_sha(&self) -> Sha {
        self.sha.clone()
    }
}

impl<'a> GitHubProperties<'a> for HandleCommit<'a> {
    type Content = Commit;
    type Parent = HandleRepository<'a>;
    
    fn get_client(&'a self) -> &'a Client {
        self.get_parent()
            .get_client()
    }
    
    fn get_parent(&'a self) -> &'a Self::Parent {
        &(self.repository)
    }
    
    fn get_endpoint(&self) -> Cow<'a, str> {
        let HandleCommit { repository, .. } = { self };
        format!("repos/{repository}/git/commits/{self}").into()
    }
}

impl<'a> Into<Sha<'static>> for &'a HandleCommit<'a> {
    fn into(self) -> Sha<'static> {
        self.sha.to_owned()
    }
}

impl<'a> Into<Sha<'static>> for HandleCommit<'a> {
    fn into(self) -> Sha<'static> {
        self.sha.to_owned()
    }
}

impl<'a> FmtDisplay for HandleCommit<'a> {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        write!(fmt, "{sha}", sha = self.sha)
    }
}
