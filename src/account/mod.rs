use std::{

    sync::{Arc}, 
    
    fmt::{

        Formatter as FmtFormatter,
        Display as FmtDisplay,
        Result as FmtResult,
        Debug as FmtDebug,
    }, 
};

use crate::{

    repository::{

        HandleRepositoryError,
        HandleRepository,
    },

    client::{
        
        ClientError,
        Client,
    },
};

use thiserror::{Error};

use serde::{

    de::{DeserializeOwned},

    Serialize,
};

use crate::{

    account::{

        organization::{

            HandleOrganizationError,
            HandleOrganization,
        },

        user::{

            HandleUserError,
            HandleUser,
        },
    },

    models::common::user::{User},

    GitHubProperties,
    GitHubResult,
};

pub mod organization;
pub mod user;

#[derive(Error, Debug)]
pub enum AccountError {
    #[error("Client error!")]
    Client(#[from] ClientError),
    #[error("Repository error!")]
    Repository(#[from] HandleRepositoryError),
    #[error("Organization error!")]
    Organization(#[from] HandleOrganizationError),
    #[error("User error!")]
    User(#[from] HandleUserError),
    #[error("Unsupported user type: '{account}'")]
    Unsupported { account: User },
}

#[derive(Clone, Debug)]
pub enum Account {
    Organization(Arc<HandleOrganization>),
    User(Arc<HandleUser>),
}

impl Account {
    pub(crate) fn from_name(client: Client, name: impl AsRef<str>) -> GitHubResult<Account, AccountError> {
        let name = name.as_ref();

        let account = client.get(format!("users/{name}"))?
            .send()?.json()?;

        match account {
            User::Organization { name, .. } => Ok(Account::Organization(Arc::new_cyclic(|reference| {
                HandleOrganization { reference: reference.clone(), client, name }
            }))),
            User::User { name, .. } => Ok(Account::User(Arc::new_cyclic(|reference| {
                HandleUser { reference: reference.clone(), client, name }
            }))),
            _ => Err(AccountError::Unsupported { account }),
        }
    }

    pub(crate) fn get_client(&self) -> Client {
        match self {
            Account::Organization(organization) => organization.get_client(),
            Account::User(user) => user.get_client(),
        }
    }

    pub fn try_get_properties<T>(&self) -> Result<T, AccountError> 
    where T: DeserializeOwned + FmtDebug {
        match self {
            Account::Organization(organization) => Ok(organization.try_get_properties()?),
            Account::User(user) => Ok(user.try_get_properties()?),
        }
    }

    pub fn try_set_properties<T>(&self, ref payload: T) -> GitHubResult<(), AccountError>
    where T: Serialize + FmtDebug {

        match self {
            Account::Organization(organization) => {
                organization.try_set_properties(payload)?;
            },
            Account::User(user) => {
                user.try_set_properties(payload)?;
            },
        };

        Ok(())
    }

    pub fn try_get_repository(&self, name: impl AsRef<str>) -> GitHubResult<Arc<HandleRepository>, AccountError> {
        Ok(HandleRepository::try_fetch(self.clone(), name)?)
    }

    pub fn try_get_all_repositories(&self) -> GitHubResult<Vec<Arc<HandleRepository>>, AccountError> {
        Ok(HandleRepository::try_fetch_all(self.clone())?)
    }
}

impl From<Arc<HandleOrganization>> for Account {
    fn from(organization: Arc<HandleOrganization>) -> Account {
        Account::Organization(organization)
    }
}

impl From<HandleOrganization> for Account {
    fn from(organization: HandleOrganization) -> Account {
        Account::Organization(organization.into())
    }
}

impl From<Arc<HandleUser>> for Account {
    fn from(user: Arc<HandleUser>) -> Account {
        Account::User(user)
    }
}

impl From<HandleUser> for Account {
    fn from(user: HandleUser) -> Account {
        Account::User(user.into())
    }
}

impl FmtDisplay for Account {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        match self {
            Account::Organization(organization) => write!(fmt, "{organization}"),
            Account::User(user) => write!(fmt, "{user}"),
        }
    }
}