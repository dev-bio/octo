
use std::{

    sync::{Arc, Weak},
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

#[derive(Debug, Clone)]
pub struct HandleCommit {
    pub(crate) reference: Weak<HandleCommit>,
    pub(crate) repository: Arc<HandleRepository>,
    pub(crate) date: Date,
    pub(crate) sha: Sha<'static>,
}

impl HandleCommit {
    pub(crate) fn try_fetch<'a>(repository: impl Into<Arc<HandleRepository>>, commit: impl Into<Sha<'a>>) -> GitHubResult<Arc<HandleCommit>, CommitError> {
        let repository = repository.into();
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

        Ok(Arc::new_cyclic(|reference| HandleCommit {
            reference: reference.clone(),
            repository,
            date,
            sha,
        }))
    }

    pub(crate) fn try_create(repository: impl Into<Arc<HandleRepository>>, parents: impl AsRef<[HandleCommit]>, tree: Tree, message: impl AsRef<str>) -> GitHubResult<Arc<HandleCommit>, CommitError> {
        let repository = repository.into();

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

        Ok(Arc::new_cyclic(|reference| HandleCommit {
            reference: reference.clone(),
            repository,
            date,
            sha,
        }))
    }

    pub fn try_compare(&self, head: impl AsRef<HandleCommit>) -> GitHubResult<Compare, CommitError> {
        Ok(Compare::try_from_base_head(self.repository.clone(), self.get_reference(), head.as_ref())?)
    }

    pub fn try_get_parents(&self) -> GitHubResult<Vec<Arc<HandleCommit>>, CommitError> {
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

        Ok(Tree::try_fetch(repository.clone(), sha, recursive)?)
    }

    pub fn try_get_date(&self) -> GitHubResult<Date, CommitError> {
        let repository = self.repository.clone();
        let client = self.repository.get_client();

        let response = client.get(format!("repos/{repository}/git/commits/{self}"))?
            .send()?;

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
    
    pub fn get_repository(&self) -> Arc<HandleRepository> {
        self.repository.clone()
    }

    pub fn get_date(&self) -> Date {
        self.date.clone()
    }

    pub fn get_sha(&self) -> Sha {
        self.sha.clone()
    }
}

impl GitHubProperties for HandleCommit {
    type Content = Commit;
    type Parent = Arc<HandleRepository>;
    
    fn get_client(&self) -> Client {
        self.get_parent()
            .get_client()
    }
    
    fn get_parent(&self) -> Self::Parent {
        self.repository.clone()
    }
    
    fn get_endpoint(&self) -> String {
        let HandleCommit { repository, .. } = { self };
        format!("repos/{repository}/git/commits/{self}")
    }

    fn get_reference(&self) -> Arc<Self> {
        self.reference.upgrade()
            .unwrap()
    }
}

impl AsRef<HandleCommit> for HandleCommit {
    fn as_ref(&self) -> &HandleCommit { self }
}

impl FmtDisplay for HandleCommit {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        write!(fmt, "{sha}", sha = self.sha)
    }
}

impl Into<Sha<'static>> for HandleCommit {
    fn into(self) -> Sha<'static> {
        self.sha.to_owned()
    }
}