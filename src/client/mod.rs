use std::fmt::{
    
    Display as FmtDisplay,
    Debug as FmtDebug,
};

use backoff::{

    ExponentialBackoff as BackoffExponential,
    Error as BackoffError,
};

pub use bytes::{Bytes};

use reqwest::{

    header::{

        HeaderValue,
        HeaderName, 
        HeaderMap, 
    }, 

    blocking::{

        multipart::{Form}, 

        Client as ReqwestClient, 

        RequestBuilder,
        Response,
        Request,
        Body, 
    },

    Url, 
};

use secrecy::{
    
    ExposeSecret,
    Secret,
};

use http::{

    Version as HttpVersion,
    Error as HttpError, 
};

use thiserror::{Error};

use serde::{
    
    de::{DeserializeOwned},

    Deserialize,
    Serialize, 
};

use crate::{

    repository::{HandleRepository},
    
    account::{

        organization::{
        
            HandleOrganizationError,
            HandleOrganization,
        },

        user::{
        
            HandleUserError,
            HandleUser,
        },

        AccountError, 
        Account,
    },

    models::common::user::{User},

    GitHubResult, 
    GitHubError, 
};

pub type Token = Secret<String>;

#[derive(Error, Debug)]
pub enum ClientRequestError {
    #[error("Server is unavailable!")]
    Unavailable,
    #[error("Request could not be built!")]
    Build,
    #[error("Request could not be cloned!")]
    Clone,
}

#[derive(Error, Debug)]
pub enum ClientResponseError {
    #[error("Unautorized!")]
    Unauthorized { code: u16, message: Option<String> },
    #[error("Invalid user input!")]
    Validation { code: u16, message: Option<String> },
    #[error("Nothing was found!")]
    Nothing { code: u16, message: Option<String> },
    #[error("Unhandled error!")]
    Unhandled { code: u16, message: Option<String> },
    #[error("Malformed response, reason: '{reason}'")]
    Malformed { reason: String },
    #[error("Encoding error!")]
    Encoding,
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Request error!")]
    Request(#[from] ClientRequestError),
    #[error("Response error!")]
    Response(#[from] ClientResponseError),
    #[error("Failed to parse endpoint: {endpoint}")]
    ParseEndpoint { endpoint:  String },
    #[error("Initialization error!")]
    Initialize,
}

#[derive(Clone, Debug)]
pub struct Client {
    pub client: ReqwestClient,
    pub token: Option<Token>,
}

impl Client {
    /// Fetches optional token from action environment, expects input with name `github-token`.
    pub fn new() -> GitHubResult<Client, GitHubError> {
        Client::new_with_token(None::<String>)
    }

    pub fn new_with_token(token: Option<impl AsRef<str>>) -> GitHubResult<Client, GitHubError> {
        let mut headers = HeaderMap::new();

        headers.insert(HeaderName::from_static("x-github-api-version"), {
            "2022-11-28".parse()
                .unwrap()
        });

        headers.insert(HeaderName::from_static("accept"), {
            "application/vnd.github+json".parse()
                .unwrap()
        });

        let client = ReqwestClient::builder().user_agent("general-action")
            .default_headers(headers).build().map_err(|_| {
                ClientError::Initialize
            })?;

        let token = token.and_then(|token| {
            Some(Secret::new(token.as_ref()
                .to_owned()))
        });

        Ok(Client { 
            
            client, 
            token,
        })
    }

    pub fn try_get_username(&self, name: impl AsRef<str>) -> GitHubResult<User, GitHubError> {
        let name = name.as_ref();

        let user = {
            self.get(format!("users/{name}"))?
                .send()?.json()?
        };

        match user {
            user @ User::Organization { .. } |
            user @ User::Mannequin { .. } |
            user @ User::User { .. } |
            user @ User::Bot { .. } => {
                Ok(user)
            },
        }
    }

    pub fn try_get_account(&self, name: impl AsRef<str>) -> GitHubResult<Account, GitHubError> {
        let name = name.as_ref();

        Ok(Account::from_name(self.clone(), name.split_once('/')
            .map(|(owner, _)| owner).unwrap_or(name))?)
    }

    pub fn try_get_organization(&self, name: impl AsRef<str>) -> GitHubResult<HandleOrganization, GitHubError> {
        let name = name.as_ref();

        let owner = self.try_get_account(name)?;
        if let Account::Organization(organization) = owner { Ok(organization) } else {
            Err(GitHubError::Account(AccountError::Organization({
                HandleOrganizationError::Organization { 
                    account: owner.try_get_properties()?
                }
            })))
        }
    }

    pub fn try_get_user(&self, name: impl AsRef<str>) -> GitHubResult<HandleUser, GitHubError> {
        let name = name.as_ref();

        let owner = self.try_get_account(name)?;
        if let Account::User(user) = owner { Ok(user) } else {
            Err(GitHubError::Account(AccountError::User({
                HandleUserError::User { 
                    account: owner.try_get_properties()?
                }
            })))
        }
    }

    pub fn try_get_repository(&self, name: impl AsRef<str>) -> GitHubResult<HandleRepository, GitHubError> {
        Ok(self.try_get_account(name.as_ref())?.try_get_repository(name.as_ref())?)
    }

    fn build_endpoint(endpoint: impl AsRef<str>) -> GitHubResult<Url, ClientError> {
        let endpoint = endpoint.as_ref();

        if let Ok(url) = Url::parse("https://api.github.com") {
            if let Ok(url) = url.join(endpoint) {
                return Ok(url)
            }
        }
        
        Err(ClientError::ParseEndpoint {
            endpoint: endpoint.to_owned()
        })
    }

    pub fn get(&self, endpoint: impl AsRef<str>) -> GitHubResult<GitHubRequestBuilder, ClientError> {
        let endpoint = Client::build_endpoint(endpoint)?;

        Ok(match self.token {
            Some(ref token) => {
                GitHubRequestBuilder {
                    client: self.clone(),
                    inner: self.client.get(endpoint)
                        .bearer_auth(token.expose_secret()),
                }
            },
            None => {
                GitHubRequestBuilder {
                    client: self.clone(),
                    inner: self.client.get(endpoint),
                }
            }
        })
    }

    pub fn put(&self, endpoint: impl AsRef<str>) -> GitHubResult<GitHubRequestBuilder, ClientError> {
        let endpoint = Client::build_endpoint(endpoint)?;

        Ok(match self.token {
            Some(ref token) => {
                GitHubRequestBuilder {
                    client: self.clone(),
                    inner: self.client.put(endpoint)
                        .bearer_auth(token.expose_secret()),
                }
            },
            None => {
                GitHubRequestBuilder {
                    client: self.clone(),
                    inner: self.client.put(endpoint),
                }
            }
        })
    }

    pub fn post(&self, endpoint: impl AsRef<str>) -> GitHubResult<GitHubRequestBuilder, ClientError> {
        let endpoint = Client::build_endpoint(endpoint)?;

        Ok(match self.token {
            Some(ref token) => {
                GitHubRequestBuilder {
                    client: self.clone(),
                    inner: self.client.post(endpoint)
                        .bearer_auth(token.expose_secret()),
                }
            },
            None => {
                GitHubRequestBuilder {
                    client: self.clone(),
                    inner: self.client.post(endpoint),
                }
            }
        })
    }

    pub fn patch(&self, endpoint: impl AsRef<str>) -> GitHubResult<GitHubRequestBuilder, ClientError> {
        let endpoint = Client::build_endpoint(endpoint)?;

        Ok(match self.token {
            Some(ref token) => {
                GitHubRequestBuilder {
                    client: self.clone(),
                    inner: self.client.patch(endpoint)
                        .bearer_auth(token.expose_secret()),
                }
            },
            None => {
                GitHubRequestBuilder {
                    client: self.clone(),
                    inner: self.client.patch(endpoint),
                }
            }
        })
    }

    pub fn delete(&self, endpoint: impl AsRef<str>) -> GitHubResult<GitHubRequestBuilder, ClientError> {
        let endpoint = Client::build_endpoint(endpoint)?;

        Ok(match self.token {
            Some(ref token) => {
                GitHubRequestBuilder {
                    client: self.clone(),
                    inner: self.client.delete(endpoint)
                        .bearer_auth(token.expose_secret()),
                }
            },
            None => {
                GitHubRequestBuilder {
                    client: self.clone(),
                    inner: self.client.delete(endpoint),
                }
            }
        })
    }

    pub fn execute(&self, request: Request) -> GitHubResult<GitHubResponse, ClientError> {
        Ok(GitHubResponse::from(self.client.execute(request).map_err(|_| {
            ClientRequestError::Unavailable
        })?))
    }
}

pub struct GitHubRequestBuilder {
    client: Client,
    inner: RequestBuilder,
}

impl GitHubRequestBuilder {
    pub fn header<K, V>(self, key: K, value: V) -> GitHubRequestBuilder
    where <HeaderValue as TryFrom<V>>::Error: Into<HttpError>,
          <HeaderName as TryFrom<K>>::Error: Into<HttpError>,
          HeaderValue: TryFrom<V>,
          HeaderName: TryFrom<K>,
    {
        GitHubRequestBuilder {
            inner: self.inner.header(key, value),
            .. self
        }
    }

    pub fn headers(self, headers: HeaderMap) -> GitHubRequestBuilder {
        GitHubRequestBuilder {
            inner: self.inner.headers(headers),
            .. self
        }
    }

    pub fn version(self, version: HttpVersion) -> GitHubRequestBuilder {
        GitHubRequestBuilder {
            inner: self.inner.version(version),
            .. self
        }
    }

    pub fn basic_auth<U, P>(self, username: U, password: Option<P>) -> GitHubRequestBuilder
    where U: FmtDisplay, P: FmtDisplay {
        GitHubRequestBuilder {
            inner: self.inner.basic_auth(username, password),
            .. self
        }
    }

    pub fn bearer_auth<T>(self, token: T) -> GitHubRequestBuilder
    where T: FmtDisplay {
        GitHubRequestBuilder {
            inner: self.inner.bearer_auth(token),
            .. self
        }
    }

    pub fn body<T: Into<Body>>(self, body: T) -> GitHubRequestBuilder {
        GitHubRequestBuilder {
            inner: self.inner.body(body),
            .. self
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn timeout(self, timeout: std::time::Duration) -> GitHubRequestBuilder {
        GitHubRequestBuilder {
            inner: self.inner.timeout(timeout),
            .. self
        }
    }

    pub fn multipart(self, multipart: Form) -> GitHubRequestBuilder {
        GitHubRequestBuilder {
            inner: self.inner.multipart(multipart),
            .. self
        }
    }

    pub fn query<T: Serialize + ?Sized>(self, query: &T) -> GitHubRequestBuilder {
        GitHubRequestBuilder {
            inner: self.inner.query(query),
            .. self
        }
    }

    pub fn form<T: Serialize + ?Sized>(self, form: &T) -> GitHubRequestBuilder {
        GitHubRequestBuilder {
            inner: self.inner.form(form),
            .. self
        }
    }

    pub fn json<T: Serialize + ?Sized>(self, json: &T) -> GitHubRequestBuilder {
        GitHubRequestBuilder {
            inner: self.inner.json(json),
            .. self
        }
    }
   
    pub fn send(self) -> GitHubResult<GitHubResponse, ClientError> {
        let request = {
            self.inner.build().map_err(|_| {
                ClientRequestError::Build
            })?
        };

        let client = self.client.clone();
        let response = backoff::retry(BackoffExponential::default(), move || {
            if let Some(request) = request.try_clone() {
                return client.execute(request).map_err(|error| {
                    BackoffError::transient(error)
                })
            }

            Err(BackoffError::transient(ClientError::Request({
                ClientRequestError::Clone
            })))
            
        }).map_err(|error| match error {
            BackoffError::Transient { err, .. } => err,
            BackoffError::Permanent(err) => err,
        })?;

        if response.is_success() { 
            Ok(response) 
        } 
        
        else {

            #[derive(Default, Debug)]
            #[derive(Deserialize)]
            struct Capsule {
                message: Option<String>,
            }

            let code = response.code();
            let Capsule { message } = response.json()
                .unwrap_or_default();

            match code {
                401 | 403 => Err(ClientError::Response({
                    ClientResponseError::Unauthorized { 
                        code, message 
                    }
                })),
                404 => Err(ClientError::Response({
                    ClientResponseError::Nothing { 
                        code, message 
                    }
                })),
                422 => Err(ClientError::Response({
                    ClientResponseError::Validation { 
                        code, message 
                    }
                })),
                _ => Err(ClientError::Response({
                    ClientResponseError::Unhandled { 
                        code, message 
                    }
                })),
            }
        }
    }
}

#[derive(Debug)]
pub struct GitHubResponse {
    inner: Response,
}

impl GitHubResponse {
    pub fn from(response: Response) -> GitHubResponse {
        GitHubResponse { inner: response }
    }

    pub fn is_success(&self) -> bool {
        self.inner.status()
            .is_success()
    }

    pub fn code(&self) -> u16 {
        self.inner.status()
            .as_u16()
    }

    pub fn bytes(self) -> GitHubResult<Bytes, ClientError> {
        let bytes = {
            self.inner.bytes().map_err(|_| {
                ClientResponseError::Encoding
            })?
        };

        Ok(bytes)
    }

    pub fn text(self) -> GitHubResult<String, ClientError> {
        let text = {
            self.inner.text().map_err(|_| {
                ClientResponseError::Encoding
            })?
        };

        Ok(text)
    }

    pub fn json<T: DeserializeOwned + FmtDebug>(self) -> GitHubResult<T, ClientError> {
        let ref notation = {
            self.inner.text().map_err(|_| {
                ClientResponseError::Encoding
            })?
        };

        Ok(serde_json::from_str(notation).map_err(|error| {
            ClientResponseError::Malformed { 
                reason: error.to_string() 
            }
        })?)
    }
}
