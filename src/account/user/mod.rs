use std::{

    sync::{Weak, Arc},

    fmt::{

        Formatter as FmtFormatter,
        Display as FmtDisplay,
        Result as FmtResult,
    }, 
};

use thiserror::{Error};

use crate::{

    repository::{

        HandleRepositoryError,
    },

    client::{

        ClientError,
        Client, 
    }, 

    models::common::user::{User},
    
    GitHubProperties,
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
    pub(crate) reference: Weak<HandleUser>,
    pub(crate) client: Client,
    pub(crate) name: String,
}

impl HandleUser {
    pub(crate) fn get_client(&self) -> Client {
        self.client.clone()
    }
}

impl GitHubProperties for HandleUser {
    type Content = User;
    type Parent = Client;

    fn get_client(&self) -> Client {
        self.client.clone()
    }

    fn get_parent(&self) -> Self::Parent {
        self.client.clone()
    }
    
    fn get_endpoint(&self) -> String {
        format!("users/{self}")
    }

    fn get_reference(&self) -> Arc<Self> {
        self.reference.upgrade()
            .expect("HandleUser reference is dangling!")
    }
}

impl FmtDisplay for HandleUser {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        let HandleUser { name, .. } = { self };
        write!(fmt, "{name}")
    }
}