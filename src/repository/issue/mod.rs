use std::fmt::{
    
    Formatter as FmtFormatter,
    Display as FmtDisplay,
    Result as FmtResult,
};

use crate::{

    repository::{

        issue::{

            comment::{
    
                IssueCommentError,
                HandleIssueComment,
            },
        },

        HandleRepository,
    },

    client::{

        ClientError,
        Client,
    },
    
    models::common::{
        
        issue::{Issue},
        user::{User},
    },

    GitHubProperties,
    GitHubEndpoint,
    GitHubObject,
    GitHubResult, 
    Number,
};

use serde::{Deserialize};

use thiserror::{Error};

mod comment;

#[derive(Error, Debug)]
pub enum IssueError {
    #[error("Client error!")]
    Client(#[from] ClientError),
    #[error("Issue comment error!")]
    Comment(#[from] IssueCommentError),
    #[error("Not an issue: {number}")]
    Issue { number: Number },
    #[error("Failed to fetch issue author: '{author}'")]
    Author { author: String },
    #[error("Failed to fetch issue assignee: '{assignee}'")]
    Assignee { assignee: String },
}

#[derive(Clone, Debug)]
pub struct HandleIssue {
    repository: HandleRepository,
    number: Number,
}

impl HandleIssue {
    pub(crate) fn try_fetch(repository: HandleRepository, number: Number) -> GitHubResult<HandleIssue, IssueError> {
        let client = repository.get_client();

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct CapsulePullRequest {
            // ..
        }

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            pull_request: Option<CapsulePullRequest>,
            assignees: Option<Vec<User>>,
            #[serde(rename = "user")]
            author: User,
            number: Number,
        }

        let issue: Issue = {
            client.get(format!("repos/{repository}/issues/{number}"))?
                .send()?.json()?
        };

        if issue.is_pull_request() {
            return Err(IssueError::Issue { number });
        }

        Ok(HandleIssue {

            repository,
            number,
        })
    }

    pub(crate) fn try_fetch_all(repository: HandleRepository) -> GitHubResult<Vec<HandleIssue>, IssueError> {
        let client = repository.get_client();

        let mut collection = Vec::new();
        let mut page = 0;

        loop {

            page = { page + 1 };

            let capsules: Vec<Issue> = {
                let ref query = [
                    ("per_page", 100),
                    ("page", page),
                ];
                client.get(format!("repos/{repository}/issues"))?
                    .query(query).send()?.json()?
            };

            collection.extend_from_slice({
                capsules.as_slice()
            });

            if capsules.len() < 100 {
                break
            }
        }

        let mut issues = Vec::new();
        for issue in collection {
            if issue.is_pull_request() { 
                continue 
            }

            issues.push(HandleIssue { 
                
                repository: repository.clone(),
                number: issue.get_number(), 
            });
        }

        Ok(issues)
    }

    pub fn try_get_comment(&self, number: Number) -> GitHubResult<HandleIssueComment, IssueError> {
        Ok(HandleIssueComment::try_fetch(self.clone(), number)?)
    }

    pub fn try_has_comment(&self, number: Number) -> GitHubResult<bool, IssueError> {
        match HandleIssueComment::try_fetch(self.clone(), number) {
            Err(IssueCommentError::Nothing { .. }) => Ok(false),
            Err(error) => Err(IssueError::Comment(error)),
            Ok(_) => Ok(true),
        }
    }

    pub fn try_get_all_issue_comments(&self) -> GitHubResult<Vec<HandleIssueComment>, IssueError> {
        Ok(HandleIssueComment::try_fetch_all(self.clone())?)
    }

    pub fn try_has_comments(&self) -> GitHubResult<bool, IssueError> {
        match HandleIssueComment::try_fetch_all(self.clone()) {
            Err(IssueCommentError::Nothing { .. }) => Ok(false),
            Err(error) => Err(IssueError::Comment(error)),
            Ok(_) => Ok(true),
        }
    }

    pub fn try_create_comment(&self, content: impl AsRef<str>) -> GitHubResult<HandleIssueComment, IssueError> {
        Ok(HandleIssueComment::try_create(self.clone(), content.as_ref())?)
    }

    pub fn try_delete_comment(&self, number: Number) -> GitHubResult<(), IssueError> {
        Ok(HandleIssueComment::try_delete(self.clone(), number)?)
    }
}

impl GitHubEndpoint for HandleIssue {
    fn get_client(&self) -> Client {
        self.repository.get_client()
    }

    fn get_endpoint(&self) -> String {
        let HandleIssue { repository, .. } = { self };
        format!("repos/{repository}/issues/{self}")
    }
}

impl GitHubObject for HandleIssue {
    fn get_number(&self) -> Number {
        self.number.clone()
    }
}

impl GitHubProperties for HandleIssue {
    type Content = Issue;
    type Parent = HandleRepository;

    fn get_parent(&self) -> Self::Parent {
        self.repository.clone()
    }
}

impl FmtDisplay for HandleIssue {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        write!(fmt, "{number}", number = {
            self.number.clone()
        })
    }
}
