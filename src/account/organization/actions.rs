use anyhow::{Result};

use super::{HandleOrganization};
use crate::{GitHubProperties};

#[derive(Debug)]
pub struct HandleActions<'a> {
    pub(crate) organization: &'a HandleOrganization<'a>,
}

impl<'a> HandleActions<'a> {
    pub(crate) fn from(organization: &'a HandleOrganization<'a>) -> HandleActions<'a> {
        HandleActions { organization }
    }

    pub fn try_set_allow_list<P: AsRef<str>>(&'a self, set: impl AsRef<[P]>) -> Result<&'a HandleActions<'a>> {
        use model::{AllowedActions};

        let AllowedActions { verified, native, .. } = AllowedActions::from({
            self.organization
        })?;

        let mut list: Vec<String> = set.as_ref().iter()
            .map(|item| item.as_ref().to_owned())
            .collect();

        list.sort();
        list.dedup();

        let ref payload = AllowedActions {
            verified,
            native,
            list,
        };

        let HandleActions { organization, .. } = { self };

        organization.get_client()
            .put(format!("orgs/{organization}/actions/permissions/selected-actions"))?
            .json(payload).send()?;

        Ok(self)
    }

    pub fn try_add_allow_list<P: AsRef<str>>(&'a self, add: impl AsRef<[P]>) -> Result<&'a HandleActions<'a>> {
        use model::{AllowedActions};

        let AllowedActions { verified, native, mut list } = AllowedActions::from({
            self.organization
        })?;

        list.extend(add.as_ref().iter().map(|item| {
            item.as_ref().to_owned()
        }));

        list.sort();
        list.dedup();

        let ref payload = AllowedActions {
            verified,
            native,
            list,
        };

        let HandleActions { organization, .. } = { self };

        organization.get_client()
            .put(format!("orgs/{organization}/actions/permissions/selected-actions"))?
            .json(payload).send()?;

        Ok(self)
    }

    pub fn try_get_allow_list(&self) -> Result<Vec<String>> {
        use model::{AllowedActions};

        let AllowedActions { list, .. } = AllowedActions::from({
            self.organization
        })?;

        Ok(list)
    }

    pub fn try_set_allow_native(&self, native: bool) -> Result<&Self> {
        use model::{AllowedActions};

        let AllowedActions { verified, list, .. } = AllowedActions::from({
            self.organization
        })?;


        let ref payload = AllowedActions {
            verified,
            native,
            list,
        };

        let HandleActions { organization, .. } = { self };

        organization.get_client()
            .put(format!("orgs/{organization}/actions/permissions/selected-actions"))?
            .json(payload).send()?;

        Ok(self)
    }

    pub fn try_get_allow_native(&self) -> Result<bool> {
        use model::{AllowedActions};

        let AllowedActions { native, .. } = AllowedActions::from({
            self.organization
        })?;

        Ok(native)
    }

    pub fn try_set_allow_verified(&'a self, verified: bool) -> Result<&'a HandleActions<'a>> {
        use model::{AllowedActions};

        let AllowedActions { native, list, .. } = AllowedActions::from({
            self.organization
        })?;

        let ref payload = AllowedActions {
            verified,
            native,
            list,
        };

        let HandleActions { organization, .. } = { self };

        let _ = {

            organization.get_client()
                .put(format!("orgs/{organization}/actions/permissions/selected-actions"))?
                .json(payload)
                .send()?
        };

        Ok(self)
    }

    pub fn try_get_allow_verified(&self) -> Result<bool> {
        use model::{AllowedActions};

        let AllowedActions { verified, .. } = AllowedActions::from({
            self.organization
        })?;

        Ok(verified)
    }
}

mod model {

    use anyhow::{Result};
    use serde::{

        Deserialize,
        Serialize, 
    };

    use crate::account::organization::{HandleOrganization};
    use crate::{GitHubProperties};

    #[derive(Clone, Debug)]
    #[derive(Serialize, Deserialize)]
    pub struct AllowedActions {
        #[serde(rename = "verified_allowed")]
        pub(super) verified: bool,
        #[serde(rename = "github_owned_allowed")]
        pub(super) native: bool,
        #[serde(rename = "patterns_allowed")]
        pub(super) list: Vec<String>,
    }

    impl AllowedActions {
        pub(super) fn from<'a>(organization: &'a HandleOrganization<'a>) -> Result<Self> {
            let response = {
                
                organization.get_client()
                    .get(format!("orgs/{organization}/actions/permissions/selected-actions"))?
                    .send()?.json()?
            };

            Ok(response)
        }
    }
}