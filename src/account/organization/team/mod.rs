use std::fmt::{

    Formatter as FmtFormatter,
    Display as FmtDisplay,
    Result as FmtResult,
    Debug as FmtDebug,
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
    
    GitHubResult, GitHubProperties, GitHubEndpoint, Number, GitHubObject, 
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
pub struct HandleTeam {
    pub(crate) organization: HandleOrganization,
    pub(crate) number: Number,
    pub(crate) slug: String,
}

impl HandleTeam {
    pub(crate) fn try_fetch(organization: HandleOrganization, slug: impl AsRef<str>) -> GitHubResult<HandleTeam, HandleTeamError> {
        let slug = slug.as_ref()
            .to_owned();

        let Team { number, slug, .. } = {
            organization.get_client()
                .get(format!("orgs/{organization}/teams/{slug}"))?
                .send()?.json()?
        };

        Ok(HandleTeam { 
            organization,
            number,
            slug,
        })
    }

    pub(crate) fn try_fetch_all(organization: HandleOrganization) -> GitHubResult<Vec<HandleTeam>, HandleTeamError> {
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

        Ok(collection.into_iter().map(|Team { number, slug, .. }| HandleTeam { 
            organization: organization.clone(), number: number.clone(), slug: slug.clone()
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

impl GitHubObject for HandleTeam {
    fn get_number(&self) -> Number {
        self.number.clone()
    }
}

impl GitHubEndpoint for HandleTeam {
    fn get_client(&self) -> Client {
        self.organization.get_client()
    }

    fn get_endpoint(&self) -> String {
        let HandleTeam { organization, slug, .. } = { self };
        format!("orgs/{organization}/teams/{slug}")
    }
}

impl GitHubProperties for HandleTeam {
    type Content = Team;
    type Parent = HandleOrganization;

    fn get_parent(&self) -> Self::Parent {
        self.organization.clone()
    }
}

impl FmtDisplay for HandleTeam {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        let HandleTeam { slug, .. } = { self };
        write!(fmt, "{slug}")
    }
}