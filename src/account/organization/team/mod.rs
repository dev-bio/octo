use std::{

    borrow::{Cow},

    fmt::{

        Formatter as FmtFormatter,
        Display as FmtDisplay,
        Result as FmtResult,
        Debug as FmtDebug,
    },
};

use thiserror::{Error};

use serde::de::{DeserializeOwned};

use crate::{
    
    repository::{HandleRepositoryError},

    client::{

        ClientError,
        Client, 
    },

    models::common::{

        user::{User}, 
        team::{Team},
    },
    
    GitHubProperties,
    GitHubResult, 
};

use super::{HandleOrganization};

#[derive(Error, Debug)]
pub enum HandleTeamError {
    #[error("Client error!")]
    Client(#[from] ClientError),
    #[error("Repository error!")]
    Repository(#[from] HandleRepositoryError),
    #[error("Not an organization, got: '{account:?}'")]
    Organization { account: User },
}

#[derive(Clone, Debug)]
pub struct HandleTeam<'a> {
    pub(crate) organization: &'a HandleOrganization<'a>,
    pub(crate) slug: String,
}

impl<'a> HandleTeam<'a> {
    pub(crate) fn try_fetch(organization: &'a HandleOrganization, slug: impl AsRef<str>) -> GitHubResult<HandleTeam<'a>, HandleTeamError> {
        let slug = slug.as_ref()
            .to_owned();

        let Team { slug, .. } = {
            organization.get_client()
                .get(format!("orgs/{organization}/teams/{slug}"))?
                .send()?.json()?
        };

        Ok(HandleTeam { 
            organization,
            slug,
        })
    }

    pub(crate) fn try_fetch_all(organization: &'a HandleOrganization) -> GitHubResult<Vec<HandleTeam<'a>>, HandleTeamError> {
        let client = organization.get_client();
        
        let mut collection = Vec::new();
        let mut page = 0;
        

        loop {

            page = { page + 1 };

            let capsules: Vec<Team> = {
                let ref query = [
                    ("per_page", 100),
                    ("page", page),
                ];

                client.get(format!("orgs/{organization}/teams"))?
                    .query(query).send()?.json()?
            };

            collection.extend_from_slice({
                capsules.as_slice()
            });

            if capsules.len() < 100 {
                break
            }
        }

        Ok(collection.into_iter()
            .map(|Team { slug, .. }| HandleTeam { 
                organization, slug 
            }).collect())
    }

    pub fn try_has_team_member<T>(&self, ref member: T) -> GitHubResult<bool, HandleTeamError>
    where T: DeserializeOwned + FmtDebug + PartialEq {
        let members: Vec<T> = {
            self.try_get_team_members()?
        };

        Ok(members.contains(member))
    }

    pub fn try_get_team_members<T>(&self) -> GitHubResult<Vec<T>, HandleTeamError> 
    where T: DeserializeOwned + FmtDebug {
        let organization = self.get_parent();
        let client = self.get_client();

        Ok(client.get(format!("orgs/{organization}/teams/{self}/members"))?
            .send()?.json()?)
    }
}

impl<'a> GitHubProperties<'a> for HandleTeam<'a> {
    type Content = Team;
    type Parent = HandleOrganization<'a>;
    
    fn get_client(&self) -> &'a Client {
        self.organization.get_client()
    }
    
    fn get_parent(&self) -> &'a Self::Parent {
        self.organization
    }
    
    fn get_endpoint(&self) -> Cow<'a, str> {
        let HandleTeam { organization, .. } = { self };
        format!("orgs/{organization}/teams/{self}").into()
    }
}

impl<'a> FmtDisplay for HandleTeam<'a> {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        let HandleTeam { slug, .. } = { self };
        write!(fmt, "{slug}")
    }
}