use std::{
    
    borrow::{Cow}, 

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
pub enum Account<'a> {
    Organization(HandleOrganization<'a>),
    User(HandleUser<'a>),
}

impl<'a> Account<'a> {
    pub(crate) fn try_from_name(client: Client, name: impl Into<Cow<'a, str>>) -> GitHubResult<Account<'a>, AccountError> {
        let name = name.into();

        let account = client.get(format!("users/{name}"))?
            .send()?.json()?;

        match account {
            User::Organization { .. } => Ok(Account::Organization({
                HandleOrganization { client, name }
            })),
            User::User { .. } => Ok(Account::User({
                HandleUser { client, name }
            })),
            _ => Err(AccountError::Unsupported { account }),
        }
    }

    pub(crate) fn get_client(&'a self) -> &'a Client {
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

    pub fn try_get_repository(&'a self, name: impl Into<Cow<'a, str>>) -> GitHubResult<HandleRepository<'a>, AccountError> {
        Ok(HandleRepository::try_fetch(self.clone(), name)?)
    }

    pub fn try_get_all_repositories(&'a self) -> GitHubResult<Vec<HandleRepository>, AccountError> {
        Ok(HandleRepository::try_fetch_all(self.clone())?)
    }
}

impl<'a> From<HandleOrganization<'a>> for Account<'a> {
    fn from(organization: HandleOrganization<'a>) -> Account<'a> {
        Account::Organization(organization.into())
    }
}

impl<'a> From<HandleUser<'a>> for Account<'a> {
    fn from(user: HandleUser<'a>) -> Account<'a> {
        Account::User(user.into())
    }
}

impl<'a> FmtDisplay for Account<'a> {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        match self {
            Account::Organization(organization) => write!(fmt, "{organization}"),
            Account::User(user) => write!(fmt, "{user}"),
        }
    }
}