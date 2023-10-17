use std::{

    sync::{Arc, Weak},

    fmt::{
    
        Formatter as FmtFormatter,
        Display as FmtDisplay,
        Result as FmtResult,
    }, 
};

use crate::{

    client::{Client, ClientError, ClientResponseError},
    account::{Account},
    
    repository::{

        reference::{
            
            ReferenceError,
            HandleReference,
        },

        commit::{

            CommitError,
            HandleCommit,
        },

        issue::{

            IssueError,
            HandleIssue,
        },
        
        tree::{
    
            TreeError,
            TreeEntry,
            Tree, 
        },

        blob::{

            BlobError,
            Blob,
        },

        sha::{Sha},
    }, 
    
    models::common::repository::{Repository},
    
    GitHubProperties,
};

use serde::{

    Deserialize,
    Serialize,
};

use thiserror::{Error};
use zip::result::{ZipError};

pub mod properties;
pub mod reference;
pub mod commit;
pub mod issue;
pub mod tree;
pub mod blob;
pub mod sha;

use crate::{GitHubResult};

#[derive(Error, Debug)]
pub enum HandleRepositoryError {
    #[error("Client error!")]
    Client(#[from] ClientError),
    #[error("Reference error!")]
    Reference(#[from] ReferenceError),
    #[error("Commit error!")]
    Commit(#[from] CommitError),
    #[error("Issue error!")]
    Issue(#[from] IssueError),
    #[error("Blob error!")]
    Blob(#[from] BlobError),
    #[error("Tree error!")]
    Tree(#[from] TreeError),
    #[error("Invalid reference: '{name}'")]
    InvalidReference { name: String },
    #[error("Invalid branch: '{name}'")]
    InvalidBranch { name: String },
    #[error("Invalid tag: '{name}'")]
    InvalidTag { name: String },
    #[error("Failed to get default branch: '{name}'")]
    DefaultBranch { name: String },
    #[error("Extraction error!")]
    Archive(#[from] ZipError),
}

#[derive(Clone, Debug)]
pub struct HandleRepository {
    pub(crate) reference: Weak<HandleRepository>,
    pub(crate) owner: Arc<Account>,
    pub(crate) name: String,
}

impl HandleRepository {
    pub fn try_fetch(account: impl Into<Arc<Account>>, name: impl AsRef<str>) -> GitHubResult<Arc<HandleRepository>, HandleRepositoryError> {
        let owner = account.into();
        let name = name.as_ref();

        let components: Vec<_> = name.split('/')
            .collect();

        let response = match components.as_slice() {
            [_, name, _, ..] => owner.get_client().get(format!("repos/{owner}/{name}"))?.send()?,
            [_, name, ..] => owner.get_client().get(format!("repos/{owner}/{name}"))?.send()?,
            [name, ..] => owner.get_client().get(format!("repos/{owner}/{name}"))?.send()?,
            _ => owner.get_client().get(format!("repos/{owner}/{name}"))?.send()?,
        };

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            name: String
        }

        let Capsule { name } = {
            response.json()?
        };

        Ok(Arc::new_cyclic(|reference| HandleRepository {
            owner, name, reference: {
                reference.clone()
            }
        }))
    }

    pub fn try_fetch_all(owner: impl Into<Arc<Account>>) -> GitHubResult<Vec<Arc<HandleRepository>>, HandleRepositoryError> {
        let owner = owner.into();

        #[derive(Clone, Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            name: String,
        }

        let mut collection = Vec::new();
        let mut page = 0;

        loop {

            page = { page + 1 };

            let capsules: Vec<Capsule> = {
                let ref query = [
                    ("per_page", 100),
                    ("page", page),
                ];

                owner.get_client()
                    .get(format!("users/{owner}/repos"))?
                    .query(query).send()?.json()?
            };

            collection.extend_from_slice({
                capsules.as_slice()
            });

            if capsules.len() < 100 {
                break
            }
        }

        Ok(collection.into_iter().map(|Capsule { name }| {
            Arc::new_cyclic(|reference| HandleRepository {
                owner: owner.clone(), name, reference: {
                    reference.clone()
                }
            })
        }).collect())
    }

    pub fn try_submit_dependency_snapshot(&self, ref payload: impl Serialize) -> GitHubResult<(), HandleRepositoryError> {
        self.get_client().post(format!("repos/{self}/dependency-graph/snapshots"))?
            .json(payload).send()?;

        Ok(())
    }

    pub fn try_get_active_workflows(&self) -> GitHubResult<usize, HandleRepositoryError> {
        #[derive(Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            total_count: usize,
        }

        let Capsule { total_count } = {

            let ref query = [
                ("status", "in_progress")
            ];

            self.get_client()
                .get(format!("repos/{self}/actions/runs"))?
                .query(query).send()?.json()?
        };

        Ok(total_count)
    }

    pub fn try_get_issue(&self, id: usize) -> GitHubResult<Arc<HandleIssue>, HandleRepositoryError> {
        Ok(HandleIssue::try_fetch(self.get_reference(), id.clone())?)
    }

    pub fn try_get_all_issues(&self) -> GitHubResult<Vec<Arc<HandleIssue>>, HandleRepositoryError> {
        Ok(HandleIssue::try_fetch_all(self.get_reference())?)
    }

    pub fn try_has_tag(&self, tag: impl AsRef<str>) -> GitHubResult<bool, HandleRepositoryError> {
        Ok(self.try_get_some_tag(tag)?.is_some())
    }

    pub fn try_get_some_tag(&self, tag: impl AsRef<str>) -> GitHubResult<Option<HandleReference>, HandleRepositoryError> {
        let tag = tag.as_ref();

        let candidate = match HandleReference::try_parse(self.get_reference(), tag) {
            Ok(reference) => reference, _ => HandleReference::try_parse(self.get_reference(), {
                format!("tags/{tag}")
            })?,
        };
        
        match self.try_get_some_reference(candidate.to_string())? {
            Some(tag @ HandleReference::Tag { .. }) => Ok(Some(tag)),
            None => Ok(None), _ => Err(HandleRepositoryError::InvalidTag { 
                name: tag.to_owned() 
            })
        }
    }

    pub fn try_get_tag(&self, tag: impl AsRef<str>) -> GitHubResult<HandleReference, HandleRepositoryError> {
        let tag = tag.as_ref();

        let candidate = match HandleReference::try_parse(self.get_reference(), tag) {
            Err(_) => HandleReference::try_parse(self.get_reference(), {
                format!("tags/{tag}")
            })?,
            Ok(reference) => {
                reference
            },
        };

        match self.try_get_reference(candidate.to_string()) {
            Ok(tag @ HandleReference::Tag { .. }) => Ok(tag),
            _ => Err(HandleRepositoryError::InvalidTag { 
                name: tag.to_owned() 
            })
        }
    }

    pub fn try_has_branch(&self, branch: impl AsRef<str>) -> GitHubResult<bool, HandleRepositoryError> {
        Ok(self.try_get_some_branch(branch)?.is_some())
    }

    pub fn try_get_some_branch(&self, branch: impl AsRef<str>) -> GitHubResult<Option<HandleReference>, HandleRepositoryError> {
        let branch = branch.as_ref();

        let candidate = match HandleReference::try_parse(self.get_reference(), branch) {
            Ok(reference) => reference, _ => HandleReference::try_parse(self.get_reference(), {
                format!("heads/{branch}")
            })?,
        };
        
        match self.try_get_some_reference(candidate.to_string())? {
            Some(branch @ HandleReference::Branch { .. }) => Ok(Some(branch)),
            None => Ok(None), _ => Err(HandleRepositoryError::InvalidBranch { 
                name: branch.to_owned() 
            })
        }
    }

    pub fn try_get_branch(&self, branch: impl AsRef<str>) -> GitHubResult<HandleReference, HandleRepositoryError> {
        let branch = branch.as_ref();

        let candidate = match HandleReference::try_parse(self.get_reference(), branch) {
            Err(_) => HandleReference::try_parse(self.get_reference(), {
                format!("heads/{branch}")
            })?,
            Ok(reference) => {
                reference
            },
        };

        match self.try_get_reference(candidate.to_string()) {
            Ok(branch @ HandleReference::Branch { .. }) => Ok(branch),
            _ => Err(HandleRepositoryError::InvalidBranch { 
                name: branch.to_owned() 
            })
        }
    }

    pub fn try_get_default_branch(&self) -> GitHubResult<HandleReference, HandleRepositoryError> {
        #[derive(Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            default_branch: String,
        }

        let Capsule { default_branch } = self.try_get_properties()?;

        Ok(self.try_get_branch(default_branch.as_str()).map_err(|_| {
            HandleRepositoryError::DefaultBranch { 
                name: default_branch.to_owned() 
            }
        })?)
    }

    pub fn try_has_reference(&self, reference: impl AsRef<str>) -> GitHubResult<bool, HandleRepositoryError> {
        Ok(self.try_get_some_reference(reference)?.is_some())
    }

    pub fn try_get_some_reference(&self, reference: impl AsRef<str>) -> GitHubResult<Option<HandleReference>, HandleRepositoryError> {
        match HandleReference::try_fetch(self.get_reference(), reference) {
            Err(ReferenceError::Nothing { .. }) => Ok(None),
            Err(error) => Err(error.into()),
            Ok(ok) => Ok(Some(ok)),
        }
    }

    pub fn try_get_reference(&self, reference: impl AsRef<str>) -> GitHubResult<HandleReference, HandleRepositoryError> {
        Ok(HandleReference::try_fetch(self.get_reference(), reference)?)
    }

    pub fn try_create_tag(&self, tag: impl AsRef<str>, commit: HandleCommit) -> GitHubResult<HandleReference, HandleRepositoryError> {
        let tag = tag.as_ref();

        let reference = HandleReference::try_create(self.get_reference(), commit, {
            format!("tags/{tag}")
        })?;
        
        if reference.is_tag() { Ok(reference) } else { 
            Err(HandleRepositoryError::InvalidTag {
                name: tag.to_owned()
            })
        }
    }

    pub fn try_create_branch(&self, branch: impl AsRef<str>, commit: HandleCommit) -> GitHubResult<HandleReference, HandleRepositoryError> {
        let branch = branch.as_ref();

        let reference = HandleReference::try_create(self.get_reference(), commit, {
            format!("heads/{branch}")
        })?;
        
        if reference.is_branch() { Ok(reference) } else { 
            Err(HandleRepositoryError::InvalidBranch {
                name: branch.to_owned()
            })
        }
    }

    pub fn try_create_reference(&self, reference: impl AsRef<str>, commit: HandleCommit) -> GitHubResult<HandleReference, HandleRepositoryError> {
        Ok(HandleReference::try_create(self.get_reference(), commit, reference)?)
    }

    pub fn try_delete_tag(&self, tag: HandleReference) -> GitHubResult<(), HandleRepositoryError> {
        if tag.is_tag() { Ok(tag.try_delete()?) } else {
            Err(HandleRepositoryError::InvalidTag {
                name: tag.to_string()
            })
        }
    }

    pub fn try_delete_branch(&self, branch: HandleReference) -> GitHubResult<(), HandleRepositoryError> {
        if branch.is_branch() { Ok(branch.try_delete()?) } else {
            Err(HandleRepositoryError::InvalidBranch {
                name: branch.to_string()
            })
        }
    }

    pub fn try_delete_reference(&self, reference: HandleReference) -> GitHubResult<(), HandleRepositoryError> {
        Ok(reference.try_delete()?)
    }
    
    pub fn try_get_blob<'a>(&self, sha: impl Into<Sha<'a>>) -> GitHubResult<Blob, HandleRepositoryError> {
        Ok(Blob::try_fetch(self.reference.upgrade().unwrap(), sha)?)
    }

    pub fn try_create_binary_blob(&self, content: impl AsRef<[u8]>) -> GitHubResult<Blob, HandleRepositoryError> {
        Ok(Blob::try_create_binary_blob(self.get_reference(), content)?)
    }

    pub fn try_create_text_blob(&self, content: impl AsRef<str>) -> GitHubResult<Blob, HandleRepositoryError> {
        Ok(Blob::try_create_text_blob(self.get_reference(), content)?)
    }   

    pub fn try_get_tree<'a>(&self, sha: impl Into<Sha<'a>>, recursive: bool) -> GitHubResult<Tree, HandleRepositoryError> {
        Ok(Tree::try_fetch(self.get_reference(), sha, recursive)?)
    }

    pub fn try_create_tree(&self, entries: impl AsRef<[TreeEntry]>) -> GitHubResult<Tree, HandleRepositoryError> {
        Ok(Tree::try_create(self.get_reference(), entries)?)
    }

    pub fn try_create_tree_with_base(&self, base: impl Into<Arc<HandleCommit>>, entries: impl AsRef<[TreeEntry]>) -> GitHubResult<Tree, HandleRepositoryError> {
        Ok(Tree::try_create_with_base(self.get_reference(), base, entries)?)
    }

    pub fn try_get_commit<'a>(&self, commit: impl Into<Sha<'a>>) -> GitHubResult<Arc<HandleCommit>, HandleRepositoryError> {
        Ok(HandleCommit::try_fetch(self.get_reference(), commit)?)
    }

    pub fn try_has_commit<'a>(&self, commit: impl Into<Sha<'a>>) -> GitHubResult<bool, HandleRepositoryError> {
        match HandleCommit::try_fetch(self.get_reference(), commit) {
            Err(CommitError::Client(ClientError::Response(ClientResponseError::Nothing { .. }))) => Ok(false),
            Err(error) => Err(error.into()),
            Ok(_) => Ok(true),
        }
    }

    pub fn try_create_commit<H>(&self, parents: impl AsRef<[HandleCommit]>, tree: Tree, message: impl AsRef<str>) -> GitHubResult<Arc<HandleCommit>, HandleRepositoryError>
    where H: AsRef<HandleCommit> { Ok(HandleCommit::try_create(self.get_reference(), parents, tree, message)?) }
}

impl GitHubProperties for HandleRepository {
    type Content = Repository;
    type Parent = Arc<Account>;

    fn get_client(&self) -> Client {
        self.get_parent()
            .get_client()
    }
    
    fn get_parent(&self) -> Self::Parent {
        self.owner.clone()
    }
    
    fn get_endpoint(&self) -> String {
        format!("repos/{self}")
    }

    fn get_reference(&self) -> Arc<Self> {
        self.reference.upgrade()
            .unwrap()
    }
}

impl FmtDisplay for HandleRepository {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        write!(fmt, "{owner}/{name}", owner = self.owner, name = self.name)
    }
}