use std::{

    borrow::{Cow}, 

    fmt::{

        Formatter as FmtFormatter,
        Display as FmtDisplay,
        Result as FmtResult,
    }, 
};

use thiserror::{Error};

use serde::{Deserialize};

use crate::{
    
    repository::{HandleRepositoryError},

    client::{

        ClientError,
        Client, 
    },

    models::common::user::{User},
    
    GitHubProperties,
    GitHubResult,
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
        Ok(HandleTeam::try_fetch(self, slug.as_ref())?)
    }

    pub fn try_get_all_teams(&self) -> GitHubResult<Vec<HandleTeam>, HandleOrganizationError> {
        Ok(HandleTeam::try_fetch_all(self)?)
    }

    pub fn get_actions(&self) -> HandleActions {
        HandleActions::from(self)
    }
}

impl<'a> GitHubProperties<'a> for HandleOrganization {
    type Content = User;
    type Parent = Client;
    
    fn get_client(&'a self) -> &'a Client {
        &(self.client)
    }
    
    fn get_parent(&'a self) -> &'a Self::Parent {
        &(self.client)
    }
    
    fn get_endpoint(&'a self) -> Cow<'a, str> {
        format!("orgs/{self}").into()
    }
}

impl FmtDisplay for HandleOrganization {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        let HandleOrganization { name, .. } = { self };
        write!(fmt, "{name}")
    }
}