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

#[derive(Debug)]
pub struct Compare<'a> {
    files: Vec<CompareFile>,
    base: &'a HandleCommit<'a>,
    head: &'a HandleCommit<'a>,
}

impl<'a> Compare<'a> {
    pub fn try_from_base_head(repository: &'a HandleRepository<'a>, base: &'a HandleCommit<'a>, head: &'a HandleCommit<'a>) -> GitHubResult<Compare<'a>, CompareError> {
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

    pub fn get_base(&self) -> &'a HandleCommit<'a> {
        self.base
    }

    pub fn get_head(&self) -> &'a HandleCommit<'a>{
        self.head.clone()
    }
}

impl<'a> Deref for Compare<'a> {
    type Target = [CompareFile];

    fn deref(&self) -> &Self::Target {
        self.files.as_ref()
    }
}

impl<'a> FmtDisplay for Compare<'a> {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        write!(fmt, "{base}..{head}", base = self.base, head = self.head)
    }
}