use std::{
    
    borrow::{Cow}, 

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
pub struct HandleUser<'a> {
    pub(crate) client: Client,
    pub(crate) name: Cow<'a, str>,
}

impl<'a> GitHubProperties<'a> for HandleUser<'a> {
    type Content = User;
    type Parent = Client;

    fn get_client(&'a self) -> &'a Client {
        &(self.client)
    }

    fn get_parent(&'a self) -> &'a Self::Parent {
        &(self.client)
    }
    
    fn get_endpoint(&'a self) -> Cow<'a, str> {
        format!("users/{self}").into()
    }
}

impl<'a> FmtDisplay for HandleUser<'a> {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        let HandleUser { name, .. } = { self };
        write!(fmt, "{name}")
    }
}