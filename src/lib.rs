#![allow(unused_braces)]
#![allow(dead_code)]

use std::borrow::{Cow};
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

pub trait GitHubProperties<'a>
where Self: Sized + Clone {

    type Content: DeserializeOwned + FmtDebug;
    type Parent;

    fn get_client(&'a self) -> &'a Client;
    fn get_parent(&'a self) -> &'a Self::Parent;
    fn get_endpoint(&'a self) -> Cow<'a, str>;

    fn try_get_content(&'a self) -> GitHubResult<Self::Content, HandleRepositoryError> {
        Ok(self.get_client().get(self.get_endpoint())?
            .send()?.json()?)
    }

    fn try_get_properties<T>(&'a self) -> GitHubResult<T, HandleRepositoryError>
    where T: DeserializeOwned + FmtDebug {
        let result = {

            self.get_client()
                .get(self.get_endpoint())?
                .send()?
                .json()?
        };
    
        Ok(result)
    }

    fn try_set_properties<T>(&'a self, ref payload: T) -> GitHubResult<Self, HandleRepositoryError>
    where T: Serialize + FmtDebug {
        let _ = {

            self.get_client()
                .patch(self.get_endpoint())?
                .json(payload)
                .send()?
        };

        Ok(self.clone())
    }
}