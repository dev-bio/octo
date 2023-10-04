use std::fmt::{

    Formatter as FmtFormatter,
    Display as FmtDisplay,
    Result as FmtResult,
};

use thiserror::{Error};

use serde::{Deserialize};

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

pub mod actions;
pub mod team;

use self::{actions::{HandleActions}, team::{HandleTeamError, HandleTeam}};


#[derive(Error, Debug)]
pub enum HandleOrganizationError {
    #[error("Client error!")]
    Client(#[from] ClientError),
    #[error("Team error!")]
    Team(#[from] HandleTeamError),
    #[error("Repository error!")]
    Repository(#[from] HandleRepositoryError),
    #[error("Not an organization, got: '{account:?}'")]
    Organization { account: User },
}

#[derive(Clone, Debug)]
pub struct HandleOrganization {
    pub(crate) client: Client,
    pub(crate) name: String,
    pub(crate) number: Number,
}

impl HandleOrganization {
    pub fn try_is_verified(&self) -> GitHubResult<bool, HandleOrganizationError> {
        #[derive(Debug)]
        #[derive(Deserialize)] 
        struct Capsule { 
            is_verified: bool 
        }

        let Capsule { is_verified } = {
            self.client.get(format!("orgs/{self}"))?
                .send()?.json()?
        };

        Ok(is_verified)
    }

    pub fn try_get_team(&self, slug: impl AsRef<str>) -> GitHubResult<HandleTeam, HandleOrganizationError> {
        Ok(HandleTeam::try_fetch(self.clone(), slug.as_ref())?)
    }

    pub fn try_get_all_teams(&self) -> GitHubResult<Vec<HandleTeam>, HandleOrganizationError> {
        Ok(HandleTeam::try_fetch_all(self.clone())?)
    }

    pub fn try_get_repository(&self, name: impl AsRef<str>) -> GitHubResult<HandleRepository, HandleOrganizationError> {
        Ok(HandleRepository::try_fetch(Account::Organization(self.clone()), name)?)
    }

    pub fn try_get_all_repositories(&self) -> GitHubResult<Vec<HandleRepository>, HandleOrganizationError> {
        Ok(HandleRepository::try_fetch_all(Account::Organization(self.clone()))?)
    }

    pub fn actions(&self) -> HandleActions {
        HandleActions::from_organization(self.clone())
    }
}

impl GitHubObject for HandleOrganization {
    fn get_number(&self) -> Number {
        self.number.clone()
    }
}

impl GitHubEndpoint for HandleOrganization {
    fn get_client(&self) -> Client {
        self.client.clone()
    }

    fn get_endpoint(&self) -> String {
        format!("orgs/{self}")
    }
}

impl GitHubProperties for HandleOrganization {
    type Content = User;
    type Parent = Client;

    fn get_parent(&self) -> Self::Parent {
        self.client.clone()
    }
}

impl FmtDisplay for HandleOrganization {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        let HandleOrganization { name, .. } = { self };
        write!(fmt, "{name}")
    }
}