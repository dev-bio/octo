use std::fmt::{

    Formatter as FmtFormatter,
    Display as FmtDisplay,
    Result as FmtResult,
};

use thiserror::{Error};

use crate::{

    repository::{

        HandleRepositoryError,
        HandleRepository,
    },

    account::{Account},

    client::{

        ClientError,
        Client, 
    }, 

    models::common::user::{User},
    
    GitHubProperties,
    GitHubEndpoint,
    GitHubObject,
    GitHubResult,
    Number,
};

#[derive(Error, Debug)]
pub enum HandleUserError {
    #[error("Client error!")]
    Client(#[from] ClientError),
    #[error("Repository error!")]
    Repository(#[from] HandleRepositoryError),
    #[error("Not a user, got: '{account:?}'")]
    User { account: User },
}

#[derive(Clone, Debug)]
pub struct HandleUser {
    pub(crate) client: Client,
    pub(crate) name: String,
    pub(crate) number: Number,
}

impl HandleUser {
    pub(crate) fn get_client(&self) -> Client {
        self.client.clone()
    }

    pub fn try_get_repository(&self, name: impl AsRef<str>) -> GitHubResult<HandleRepository, HandleUserError> {
        Ok(HandleRepository::try_fetch(Account::User(self.clone()), name)?)
    }

    pub fn try_get_all_repositories(&self) -> GitHubResult<Vec<HandleRepository>, HandleUserError> {
        Ok(HandleRepository::try_fetch_all(Account::User(self.clone()))?)
    }
}

impl GitHubObject for HandleUser {
    fn get_number(&self) -> Number {
        self.number.clone()
    }
}

impl GitHubEndpoint for HandleUser {
    fn get_client(&self) -> Client {
        self.client.clone()
    }

    fn get_endpoint(&self) -> String {
        format!("users/{self}")
    }
}

impl GitHubProperties for HandleUser {
    type Content = User;
    type Parent = Client;

    fn get_parent(&self) -> Self::Parent {
        self.client.clone()
    }
}

impl FmtDisplay for HandleUser {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        let HandleUser { name, .. } = { self };
        write!(fmt, "{name}")
    }
}