#![allow(unused_braces)]
#![allow(dead_code)]

use std::fmt::{Debug as FmtDebug};

pub mod repository;
pub mod account;
pub mod client;
pub mod common;
pub mod models;

use account::{AccountError};

use client::{

    ClientError, 
    Client,
};

use repository::{HandleRepositoryError};
use thiserror::{Error};

use serde::{

    de::{DeserializeOwned},

    Serialize,
};

pub type Number = usize;

#[derive(Debug, Error)]
pub enum GitHubError {
    #[error("Account error!")]
    Account(#[from] AccountError),
    #[error("Client error!")]
    Client(#[from] ClientError),
}

pub type GitHubResult<T, E = GitHubError> = Result<T, E>;

pub trait GitHubObject {
    fn get_number(&self) -> Number;
}

pub trait GitHubEndpoint {
    fn get_client(&self) -> Client;
    fn get_endpoint(&self) -> String;
}

pub trait GitHubProperties: GitHubEndpoint
where Self: Sized + Clone {

    type Content: DeserializeOwned + FmtDebug;
    type Parent;

    fn get_parent(&self) -> Self::Parent;

    fn try_get_content(&self) -> GitHubResult<Self::Content, HandleRepositoryError> {
        Ok(self.get_client().get(self.get_endpoint())?
            .send()?.json()?)
    }

    fn try_get_properties<T>(&self) -> GitHubResult<T, HandleRepositoryError>
    where T: DeserializeOwned + FmtDebug {
        Ok(self.get_client().get(self.get_endpoint())?
            .send()?.json()?)
    }

    fn try_set_properties<T>(&self, ref payload: T) -> GitHubResult<Self, HandleRepositoryError>
    where T: Serialize + FmtDebug {
        self.get_client().patch(self.get_endpoint())?
            .json(payload).send()?;

        Ok(self.clone())
    }
}