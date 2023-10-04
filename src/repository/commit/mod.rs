
use std::{

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

        reference::{
        
            ReferenceError,
            HandleReference,
        },
        
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
    GitHubEndpoint, 
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
    Nothing { commit: Sha },
}

#[derive(Debug, Clone)]
pub struct HandleCommit {
    pub(crate) repository: HandleRepository,
    pub(crate) date: Date,
    pub(crate) sha: Sha,
}

impl HandleCommit {
    pub(crate) fn try_fetch(repository: HandleRepository, commit: impl Into<Sha>) -> GitHubResult<HandleCommit, CommitError> {
        let client = repository.get_client();
        let commit = commit.into();

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct CapsuleAuthor {
            date: Date,
        }    

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            author: CapsuleAuthor,
            sha: Sha,
        }

        let Capsule { sha, author: CapsuleAuthor { date } } = {
            let request = client.get(format!("repos/{repository}/git/commits/{commit}"))?;

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
    
    pub(crate) fn get_client(&self) -> Client {
        self.repository.get_client()
    }

    pub(crate) fn try_create(repository: HandleRepository, parents: impl AsRef<[HandleCommit]>, tree: Tree, message: impl AsRef<str>) -> GitHubResult<HandleCommit, CommitError> {
        let client = repository.get_client();

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct CapsuleAuthor {
            date: Date,
        }    

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            author: CapsuleAuthor,
            sha: Sha,
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
            
            client.post(format!("repos/{repository}/git/commits"))?
                .json(payload).send()?.json()?
        };

        Ok(HandleCommit {
            
            repository,
            date,
            sha,
        })
    }

    pub fn try_compare(&self, head: HandleCommit) -> GitHubResult<Compare, CommitError> {
        Ok(Compare::try_from_base_head(self.repository.clone(), self.clone(), head)?)
    }

    pub fn try_get_parents(&self) -> GitHubResult<Vec<HandleCommit>, CommitError> {
        let repository = self.repository.clone();

        let client = repository.get_client();
        let response = client.get(format!("repos/{repository}/git/commits/{self}"))?
            .send()?;

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct CapsuleParents {
            sha: Sha
        }

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            parents: Vec<CapsuleParents>
        }

        let Capsule { parents } = response.json()?;

        let mut collection = Vec::new();
        for CapsuleParents { sha } in parents.iter() {
            collection.push(HandleCommit::try_fetch(repository.clone(), {
                sha.clone()
            })?);
        }

        Ok(collection)
    }

    pub fn try_get_tree(&self, recursive: bool) -> GitHubResult<Tree, HandleRepositoryError> {
        let repository = self.repository.clone();
        let client = self.repository.get_client();

        let response = client.get(format!("repos/{repository}/git/commits/{self}"))?
            .send()?;

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct CapsuleTree {
            sha: Sha
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
        let repository = self.repository.clone();
        let client = self.repository.get_client();

        let cursor = Cursor::new({
            
            client.get(format!("repos/{repository}/zipball/{self}"))?
                .send()?.bytes()?
        });

        Ok(ZipArchive::new(cursor)?
            .extract(path.as_ref())?)
    }
    
    pub fn get_repository(&self) -> HandleRepository {
        self.repository.clone()
    }

    pub fn get_date(&self) -> Date {
        self.date.clone()
    }

    pub fn get_sha(&self) -> Sha {
        self.sha.clone()
    }
}

impl GitHubEndpoint for HandleCommit {
    fn get_client(&self) -> Client {
        self.repository.get_client()
    }

    fn get_endpoint(&self) -> String {
        let HandleCommit { repository, .. } = { self };
        format!("repos/{repository}/git/commits/{self}")
    }
}

impl GitHubProperties for HandleCommit {
    type Content = Commit;
    type Parent = HandleRepository;

    fn get_parent(&self) -> Self::Parent {
        self.repository.clone()
    }
}

impl FmtDisplay for HandleCommit {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        write!(fmt, "{sha}", sha = self.sha)
    }
}

impl TryFrom<HandleReference> for HandleCommit {
    type Error = HandleRepositoryError;

    fn try_from(reference: HandleReference) -> GitHubResult<Self, Self::Error> {
        Ok(reference.try_get_commit()?)
    }
}

impl Into<Sha> for HandleCommit {
    fn into(self) -> Sha {
        self.sha.clone()
    }
}