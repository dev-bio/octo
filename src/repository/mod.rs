use std::{

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
    GitHubEndpoint, 
    GitHubObject,
    Number, 
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
    owner: Account,
    name: String,
    number: Number,
}

impl HandleRepository {
    pub fn try_fetch(owner: Account, name: impl AsRef<str>) -> GitHubResult<HandleRepository, HandleRepositoryError> {
        let client = owner.get_client();
        let name = name.as_ref();

        let components: Vec<_> = name.split('/')
            .collect();

        let response = match components.as_slice() {
            [_, name, _, ..] => client.get(format!("repos/{owner}/{name}"))?.send()?,
            [_, name, ..] => client.get(format!("repos/{owner}/{name}"))?.send()?,
            [name, ..] => client.get(format!("repos/{owner}/{name}"))?.send()?,
            _ => client.get(format!("repos/{owner}/{name}"))?.send()?,
        };

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            name: String,
            id: usize,
        }

        let Capsule { name, id } = {
            response.json()?
        };

        Ok(HandleRepository {
            owner,
            name,
            number: id,
        })
    }

    pub fn try_fetch_all(owner: Account) -> GitHubResult<Vec<HandleRepository>, HandleRepositoryError> {
        let client = owner.get_client();

        #[derive(Clone, Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            name: String,
            id: usize,
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
                client.get(format!("users/{owner}/repos"))?
                    .query(query).send()?.json()?
            };

            collection.extend_from_slice({
                capsules.as_slice()
            });

            if capsules.len() < 100 {
                break
            }
        }

        Ok(collection.into_iter().map(|Capsule { id, name, .. }| HandleRepository { 
            owner: owner.clone(), name: name.clone(), number: id.clone()
        }).collect())
    }

    pub(crate) fn get_client(&self) -> Client {
        self.owner.get_client()
    }

    pub fn get_owner(&self) -> Account {
        self.owner.clone()
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_id(&self) -> usize {
        self.number.clone()
    }

    pub fn try_submit_dependency_snapshot(&self, ref payload: impl Serialize) -> GitHubResult<(), HandleRepositoryError> {
        self.get_client().post(format!("repos/{self}/dependency-graph/snapshots"))?
            .json(payload).send()?;

        Ok(())
    }

    pub fn try_get_issue(&self, id: usize) -> GitHubResult<HandleIssue, HandleRepositoryError> {
        Ok(HandleIssue::try_fetch(self.clone(), id.clone())?)
    }

    pub fn try_get_all_issues(&self) -> GitHubResult<Vec<HandleIssue>, HandleRepositoryError> {
        Ok(HandleIssue::try_fetch_all(self.clone())?)
    }

    pub fn try_has_tag(&self, tag: impl AsRef<str>) -> GitHubResult<bool, HandleRepositoryError> {
        Ok(self.try_get_some_tag(tag)?.is_some())
    }

    pub fn try_get_some_tag(&self, tag: impl AsRef<str>) -> GitHubResult<Option<HandleReference>, HandleRepositoryError> {
        let tag = tag.as_ref();

        let candidate = match HandleReference::try_parse(self.clone(), tag) {
            Ok(reference) => reference, _ => HandleReference::try_parse(self.clone(), {
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

        let candidate = match HandleReference::try_parse(self.clone(), tag) {
            Err(_) => HandleReference::try_parse(self.clone(), {
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

        let candidate = match HandleReference::try_parse(self.clone(), branch) {
            Ok(reference) => reference, _ => HandleReference::try_parse(self.clone(), {
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

        let candidate = match HandleReference::try_parse(self.clone(), branch) {
            Err(_) => HandleReference::try_parse(self.clone(), {
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
        match HandleReference::try_fetch(self.clone(), reference) {
            Err(ReferenceError::Nothing { .. }) => Ok(None),
            Err(error) => Err(error.into()),
            Ok(ok) => Ok(Some(ok)),
        }
    }

    pub fn try_get_reference(&self, reference: impl AsRef<str>) -> GitHubResult<HandleReference, HandleRepositoryError> {
        Ok(HandleReference::try_fetch(self.clone(), reference)?)
    }

    pub fn try_create_tag(&self, tag: impl AsRef<str>, commit: HandleCommit) -> GitHubResult<HandleReference, HandleRepositoryError> {
        let tag = tag.as_ref();

        let reference = HandleReference::try_create(self.clone(), commit, {
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

        let reference = HandleReference::try_create(self.clone(), commit, {
            format!("heads/{branch}")
        })?;
        
        if reference.is_branch() { Ok(reference) } else { 
            Err(HandleRepositoryError::InvalidBranch {
                name: branch.to_owned()
            })
        }
    }

    pub fn try_create_reference(&self, reference: impl AsRef<str>, commit: HandleCommit) -> GitHubResult<HandleReference, HandleRepositoryError> {
        Ok(HandleReference::try_create(self.clone(), commit, reference)?)
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
    
    pub fn try_get_blob(&self, sha: impl Into<Sha>) -> GitHubResult<Blob, HandleRepositoryError> {
        Ok(Blob::try_fetch(self.clone(), sha)?)
    }

    pub fn try_create_binary_blob(&self, content: impl AsRef<[u8]>) -> GitHubResult<Blob, HandleRepositoryError> {
        Ok(Blob::try_create_binary_blob(self.clone(), content)?)
    }

    pub fn try_create_text_blob(&self, content: impl AsRef<str>) -> GitHubResult<Blob, HandleRepositoryError> {
        Ok(Blob::try_create_text_blob(self.clone(), content)?)
    }   

    pub fn try_get_tree(&self, sha: impl Into<Sha>, recursive: bool) -> GitHubResult<Tree, HandleRepositoryError> {
        Ok(Tree::try_fetch(self.clone(), sha, recursive)?)
    }

    pub fn try_create_tree(&self, entries: impl AsRef<[TreeEntry]>) -> GitHubResult<Tree, HandleRepositoryError> {
        Ok(Tree::try_create(self.clone(), entries)?)
    }

    pub fn try_create_tree_with_base(&self, base: HandleCommit, entries: impl AsRef<[TreeEntry]>) -> GitHubResult<Tree, HandleRepositoryError> {
        Ok(Tree::try_create_with_base(self.clone(), base, entries)?)
    }

    pub fn try_get_commit(&self, commit: impl Into<Sha>) -> GitHubResult<HandleCommit, HandleRepositoryError> {
        Ok(HandleCommit::try_fetch(self.clone(), commit)?)
    }

    pub fn try_has_commit(&self, commit: impl Into<Sha>) -> GitHubResult<bool, HandleRepositoryError> {
        match HandleCommit::try_fetch(self.clone(), commit) {
            Err(CommitError::Client(ClientError::Response(ClientResponseError::Nothing { .. }))) => Ok(false),
            Err(error) => Err(error.into()),
            Ok(_) => Ok(true),
        }
    }

    pub fn try_create_commit(&self, parents: impl AsRef<[HandleCommit]>, tree: Tree, message: impl AsRef<str>) -> GitHubResult<HandleCommit, HandleRepositoryError> {
        Ok(HandleCommit::try_create(self.clone(), parents, tree, message)?)
    }
}

impl GitHubObject for HandleRepository {
    fn get_number(&self) -> Number {
        self.number.clone()
    }
}

impl GitHubEndpoint for HandleRepository {
    fn get_client(&self) -> Client {
        self.owner.get_client()
    }

    fn get_endpoint(&self) -> String {
        format!("repos/{self}")
    }
}

impl GitHubProperties for HandleRepository {
    type Content = Repository;
    type Parent = Account;

    fn get_parent(&self) -> Self::Parent {
        self.owner.clone()
    }
}

impl FmtDisplay for HandleRepository {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        write!(fmt, "{owner}/{name}", owner = self.owner, name = self.name)
    }
}