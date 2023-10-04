use std::{

    path::{PathBuf, Path},

    fmt::{
    
        Formatter as FmtFormatter,
        Display as FmtDisplay,
        Result as FmtResult,
    }, 
    
    ops::{
        
        DerefMut,
        Deref,
    },
};

use serde::{

    ser::{Serializer},
    Serialize,
    
    de::{Deserializer},
    Deserialize,
};

use thiserror::{Error};

use crate::{
    
    repository::{

        commit::{HandleCommit},
        sha::{Sha},

        HandleRepository,
    },

    client::{
        
        ClientError,
        Client,
    },

    GitHubResult,
};

use super::{HandleRepositoryError, blob::Blob};

#[derive(Debug, Clone)]
pub enum TreeEntryMode {
    File,
    Executable,
    Directory,
    Commit,
    Link,
}

impl TreeEntryMode {
    pub fn file() -> TreeEntryMode {
        TreeEntryMode::File
    }

    pub fn executable() -> TreeEntryMode {
        TreeEntryMode::Executable
    }

    pub fn directory() -> TreeEntryMode {
        TreeEntryMode::Directory
    }

    pub fn commit() -> TreeEntryMode {
        TreeEntryMode::Commit
    }

    pub fn link() -> TreeEntryMode {
        TreeEntryMode::Link
    }

    pub fn to_mode(&self) -> u32 {
        match self {
            TreeEntryMode::File => 0o100644,
            TreeEntryMode::Executable => 0o100755,
            TreeEntryMode::Directory => 0o040000,
            TreeEntryMode::Commit => 0o160000,
            TreeEntryMode::Link => 0o120000,
        }
    }
}

#[derive(Debug, Clone)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TreeEntry {
    #[serde(rename = "blob")]
    Blob {
        path: PathBuf,
        #[serde(deserialize_with = "deserialize_mode")]
        #[serde(serialize_with = "serialize_mode")]
        mode: u32,
        sha: Sha,
    },
    #[serde(rename = "tree")]
    Tree {
        path: PathBuf,
        #[serde(deserialize_with = "deserialize_mode")]
        #[serde(serialize_with = "serialize_mode")]
        mode: u32,
        sha: Sha,
    },
    #[serde(rename = "commit")]
    Commit {
        path: PathBuf,
        #[serde(deserialize_with = "deserialize_mode")]
        #[serde(serialize_with = "serialize_mode")]
        mode: u32,
        sha: Sha,
    },
}

impl TreeEntry {
    pub fn blob(blob: Blob) -> TreeEntry {
        TreeEntry::Blob { 
            path: Default::default(), 
            mode: Default::default(), 
            sha: blob.get_sha(),
        }
    }

    pub fn tree(tree: Tree) -> TreeEntry {
        TreeEntry::Tree { 
            path: Default::default(), 
            mode: Default::default(), 
            sha: tree.get_sha(),
        }
    }

    pub fn commit(sha: Sha) -> TreeEntry {
        TreeEntry::Commit { 
            path: Default::default(), 
            mode: Default::default(), 
            sha,
        }
    }

    pub fn get_path(&self) -> &Path {
        match self {
            TreeEntry::Blob { path, .. } => path.as_path(),
            TreeEntry::Tree { path, .. } => path.as_path(),
            TreeEntry::Commit { path, .. } => path.as_path(),
        }
    }

    pub fn with_mode(self, mode: TreeEntryMode) -> Self {
        match self {
            TreeEntry::Blob { path, sha, .. } => {
                TreeEntry::Blob { path, mode: mode.to_mode(), sha }
            },
            TreeEntry::Tree { path, sha, .. } => {
                TreeEntry::Tree { path, mode: mode.to_mode(), sha }
            },
            TreeEntry::Commit { path, sha, .. } => {
                TreeEntry::Commit { path, mode: mode.to_mode(), sha }
            },
        }
    }

    pub fn with_path(self, path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();

        match self {
            TreeEntry::Blob { mode, sha, .. } => {
                TreeEntry::Blob { path: path.into(), mode, sha }
            },
            TreeEntry::Tree { mode, sha, .. } => {
                TreeEntry::Tree { path: path.into(), mode, sha }
            },
            TreeEntry::Commit { mode, sha, .. } => {
                TreeEntry::Commit { path: path.into(), mode, sha }
            },
        }
    }
}

fn deserialize_mode<'de, D>(deserializer: D) -> GitHubResult<u32, D::Error>
where D: Deserializer<'de> {
    let string = String::deserialize(deserializer)?;

    use serde::de::{Error};
    u32::from_str_radix(string.as_str(), 8).map_err(|error| {
        Error::custom(error)
    })
}

fn serialize_mode<S>(mode: &u32, serializer: S) -> GitHubResult<S::Ok, S::Error>
where S: Serializer {
    serializer.serialize_str({
        dbg!(format!("{mode:06o}")
            .as_str())
    })
}

#[derive(Error, Debug)]
pub enum TreeError {
    #[error("Client error!")]
    Client(#[from] ClientError),
}

#[derive(Clone, Debug)]
pub struct Tree {
    pub(crate) repository: HandleRepository,
    pub(crate) tree: Vec<TreeEntry>,
    pub(crate) sha: Sha,
}

impl Tree {
    pub(crate) fn try_create(repository: HandleRepository, entries: impl AsRef<[TreeEntry]>) -> GitHubResult<Tree, TreeError> {
        let client = repository.get_client();

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            tree: Vec<TreeEntry>,
            sha: Sha,
        }

        let ref payload = serde_json::json!({
            "tree": entries.as_ref(),
        });

        let Capsule { tree, sha } = client.post(format!("repos/{repository}/git/trees"))?
            .json(payload).send()?.json()?;

        Ok(Tree { 

            repository,
            tree,
            sha,
        })
    }

    pub(crate) fn try_create_with_base(repository: HandleRepository, base: HandleCommit, entries: impl AsRef<[TreeEntry]>) -> GitHubResult<Tree, HandleRepositoryError> {
        let client = repository.get_client();
        let tree = base.try_get_tree(false)?;

        let ref payload = serde_json::json!({
            "base_tree": tree.get_sha(),
            "tree": entries.as_ref(),
        });

        let response = client.post(format!("repos/{repository}/git/trees"))?
            .json(payload)
            .send()?;

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            tree: Vec<TreeEntry>,
            sha: Sha,
        }

        let Capsule { tree, sha } = response.json()?;

        Ok(Tree { 

            repository,
            tree,
            sha,
        })
    }

    pub(crate) fn try_fetch(repository: HandleRepository, sha: impl Into<Sha>, recursive: bool) -> GitHubResult<Tree, TreeError> {
        let client = repository.get_client();
        let sha = sha.into();

        let ref recursive = if recursive { Vec::from([("recursive", "true")]) } else { Default::default() };
        let response = client.get(format!("repos/{repository}/git/trees/{sha}"))?
            .query(recursive).send()?;

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            tree: Vec<TreeEntry>,
            sha: Sha,
        }

        let Capsule { tree, sha } = response.json()?;

        Ok(Tree { 

            repository,
            tree,
            sha,
        })
    }

    pub(crate) fn client(&self) -> Client {
        self.repository.get_client()
    }

    pub fn get_repository(&self) -> HandleRepository {
        self.repository.clone()
    }

    pub fn get_sha(&self) -> Sha {
        self.sha.clone()
    }
}

impl Deref for Tree {
    type Target = Vec<TreeEntry>;
    fn deref(&self) -> &Self::Target {
        &self.tree
    }
}

impl DerefMut for Tree {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tree
    }
}

impl FmtDisplay for Tree {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        write!(fmt, "{sha}", sha = self.sha)
    }
}

impl Into<Sha> for Tree {
    fn into(self) -> Sha {
        self.sha.clone()
    }
}