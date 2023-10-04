use std::fmt::{

    Formatter as FmtFormatter,
    Display as FmtDisplay,
    Result as FmtResult,
    Debug as FmtDebug,
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
    Organization(HandleOrganization),
    User(HandleUser),
}

impl Account {
    pub(crate) fn from_name(client: Client, name: impl AsRef<str>) -> GitHubResult<Account, AccountError> {
        let name = name.as_ref();

        let account = client.get(format!("users/{name}"))?
            .send()?.json()?;

        match account {
            User::Organization { name, number, .. } => Ok(Account::Organization(HandleOrganization { client, name, number })),
            User::User { name, number, .. } => Ok(Account::User(HandleUser { client, name, number })),
            _ => Err(AccountError::Unsupported { account }),
        }
    }

    pub(crate) fn get_client(&self) -> Client {
        match self {
            Account::Organization(HandleOrganization { ref client, .. }) => client.clone(),
            Account::User(HandleUser { ref client, .. }) => client.clone(),
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

    pub fn try_get_repository(&self, name: impl AsRef<str>) -> GitHubResult<HandleRepository, AccountError> {
        match self {
            Account::Organization(organization) => Ok(organization.try_get_repository(name)?),
            Account::User(user) => Ok(user.try_get_repository(name)?),
        }
    }

    pub fn try_get_all_repositories(&self) -> GitHubResult<Vec<HandleRepository>, AccountError> {
        match self {
            Account::Organization(organization) => Ok(organization.try_get_all_repositories()?),
            Account::User(user) => Ok(user.try_get_all_repositories()?),
        }
    }
}

impl From<HandleOrganization> for Account {
    fn from(organization: HandleOrganization) -> Account {
        Account::Organization(organization)
    }
}

impl From<HandleUser> for Account {
    fn from(user: HandleUser) -> Account {
        Account::User(user)
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