use std::{

    path::{PathBuf},

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

use thiserror::{Error};
use serde::{Deserialize};

use crate::{

    repository::{

        commit::{HandleCommit},

        HandleRepository,
    },

    client::{ClientError},

    GitHubResult, 
};

#[derive(Debug, Clone)]
#[derive(Deserialize)]
#[serde(tag = "status")]
pub enum CompareFile {
    #[serde(rename = "added")]
    Added {
        #[serde(rename = "filename")]
        path: PathBuf,
        sha: String,
    },
    #[serde(rename = "removed")]
    Removed {
        #[serde(rename = "filename")]
        path: PathBuf,
        sha: String,
    },
    #[serde(rename = "modified")]
    Modified {
        #[serde(rename = "filename")]
        path: PathBuf,
        sha: String,
    },
    #[serde(rename = "renamed")]
    Renamed {
        #[serde(rename = "filename")]
        path: PathBuf,
        sha: String,
    },
    #[serde(rename = "copied")]
    Copied {
        #[serde(rename = "filename")]
        path: PathBuf,
        sha: String,
    },
    #[serde(rename = "changed")]
    Changed {
        #[serde(rename = "filename")]
        path: PathBuf,
        sha: String,
    },
    #[serde(rename = "unchanged")]
    Unchanged {
        #[serde(rename = "filename")]
        path: PathBuf,
        sha: String,
    },
}

#[derive(Error, Debug)]
pub enum CompareError {
    #[error("Client error!")]
    Client(#[from] ClientError),
}

#[derive(Debug, Clone)]
pub struct Compare {
    files: Vec<CompareFile>,
    base: HandleCommit,
    head: HandleCommit,
}

impl Compare {
    pub fn try_from_base_head(repository: HandleRepository, base: HandleCommit, head: HandleCommit) -> GitHubResult<Compare, CompareError> {
        let client = repository.get_client();

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            files: Vec<CompareFile>,
        }

        let Capsule { files } = client.get({
            format!("repos/{repository}/compare/{base}...{head}")
        })?.send()?.json()?;

        Ok(Compare { 

            files,
            base,
            head,
        })
    }

    pub fn files(&self) -> &[CompareFile] {
        self.files.as_slice()
    }

    pub fn get_base(&self) -> HandleCommit {
        self.base.clone()
    }

    pub fn get_head(&self) -> HandleCommit {
        self.head.clone()
    }
}

impl Deref for Compare {
    type Target = Vec<CompareFile>;

    fn deref(&self) -> &Self::Target {
        &self.files
    }
}

impl DerefMut for Compare {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.files
    }
}

impl FmtDisplay for Compare {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        write!(fmt, "{base}..{head}", base = self.base, head = self.head)
    }
}