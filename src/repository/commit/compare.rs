use std::{
    
    path::{PathBuf},
    ops::{Deref}, 

    fmt::{
    
        Formatter as FmtFormatter,
        Display as FmtDisplay,
        Result as FmtResult,
    },
    
};

use thiserror::{Error};
use serde::{Deserialize};

use crate::{

    repository::{

        commit::{HandleCommit},
        sha::{Sha},

        HandleRepository, 
    },

    client::{ClientError},

    GitHubProperties, 
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
        sha: Sha<'static>,
    },
    #[serde(rename = "removed")]
    Removed {
        #[serde(rename = "filename")]
        path: PathBuf,
        sha: Sha<'static>,
    },
    #[serde(rename = "modified")]
    Modified {
        #[serde(rename = "filename")]
        path: PathBuf,
        sha: Sha<'static>,
    },
    #[serde(rename = "renamed")]
    Renamed {
        #[serde(rename = "filename")]
        path: PathBuf,
        sha: Sha<'static>,
    },
    #[serde(rename = "copied")]
    Copied {
        #[serde(rename = "filename")]
        path: PathBuf,
        sha: Sha<'static>,
    },
    #[serde(rename = "changed")]
    Changed {
        #[serde(rename = "filename")]
        path: PathBuf,
        sha: Sha<'static>,
    },
    #[serde(rename = "unchanged")]
    Unchanged {
        #[serde(rename = "filename")]
        path: PathBuf,
        sha: Sha<'static>,
    },
}

#[derive(Error, Debug)]
pub enum CompareError {
    #[error("Client error!")]
    Client(#[from] ClientError),
}

#[derive(Clone, Debug)]
pub struct Compare {
    files: Vec<CompareFile>,
    base: HandleCommit,
    head: HandleCommit,
}

impl Compare {
    pub fn try_from_base_head(repository: &HandleRepository, base: HandleCommit, head: HandleCommit) -> GitHubResult<Compare, CompareError> {
        #[derive(Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            files: Vec<CompareFile>,
        }

        let Capsule { files } = {

            repository.get_client()
                .get(format!("repos/{repository}/compare/{base}...{head}"))?
                .send()?
                .json()?
        };

        Ok(Compare { 

            files,

            base: base.clone(),
            head: head.clone(),
        })
    }

    pub fn files(&self) -> &[CompareFile] {
        self.files.as_ref()
    }

    pub fn get_base(&self) -> HandleCommit {
        self.base.clone()
    }

    pub fn get_head(&self) -> HandleCommit{
        self.head.clone()
    }
}

impl Deref for Compare {
    type Target = [CompareFile];

    fn deref(&self) -> &Self::Target {
        self.files.as_ref()
    }
}

impl FmtDisplay for Compare {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        write!(fmt, "{base}..{head}", base = self.base, head = self.head)
    }
}