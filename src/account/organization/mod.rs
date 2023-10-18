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
pub struct HandleOrganization<'a> {
    pub(crate) client: Client,
    pub(crate) name: Cow<'a, str>,
}

impl<'a> HandleOrganization<'a> {
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

    pub fn try_get_team(&'a self, slug: impl AsRef<str>) -> GitHubResult<HandleTeam<'a>, HandleOrganizationError> {
        Ok(HandleTeam::try_fetch(self, slug.as_ref())?)
    }

    pub fn try_get_all_teams(&'a self) -> GitHubResult<Vec<HandleTeam<'a>>, HandleOrganizationError> {
        Ok(HandleTeam::try_fetch_all(self)?)
    }

    pub fn get_actions(&'a self) -> HandleActions<'a> {
        HandleActions::from(self)
    }
}

impl<'a> GitHubProperties<'a> for HandleOrganization<'a> {
    type Content = User;
    type Parent = Client;
    
    fn get_client(&'a self) -> &'a Client {
        &(self.client)
    }
    
    fn get_parent(&'a self) -> &'a Self::Parent {
        &(self.client)
    }
    
    fn get_endpoint(&self) -> Cow<'a, str> {
        format!("orgs/{self}").into()
    }
}

impl<'a> FmtDisplay for HandleOrganization<'a> {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        let HandleOrganization { name, .. } = { self };
        write!(fmt, "{name}")
    }
}