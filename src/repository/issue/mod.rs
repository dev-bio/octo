use std::{

    borrow::{Cow},
    
    fmt::{
    
        Formatter as FmtFormatter,
        Display as FmtDisplay,
        Result as FmtResult,
    }, 
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
    GitHubResult, 
    Number,
};

use serde::{Deserialize};

use thiserror::{Error};

pub mod comment;

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
    pub(crate) fn try_fetch(repository: &HandleRepository, number: Number) -> GitHubResult<HandleIssue, IssueError> {

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

            repository.get_client()
                .get(format!("repos/{repository}/issues/{number}"))?
                .send()?
                .json()?
        };

        if issue.is_pull_request() {
            return Err(IssueError::Issue { number });
        }

        Ok(HandleIssue {
            repository: repository.clone(),
            number,
        })
    }

    pub(crate) fn try_fetch_all(repository: &HandleRepository) -> GitHubResult<Vec<HandleIssue>, IssueError> {
        let mut collection = Vec::new();
        let mut page = 0;

        loop {

            page = { page + 1 };

            let capsules: Vec<Issue> = {

                let ref query = [
                    ("per_page", 100),
                    ("page", page),
                ];

                repository.get_client()
                    .get(format!("repos/{repository}/issues"))?
                    .query(query)
                    .send()?
                    .json()?
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
                repository: repository.clone(), number: {
                    issue.get_number()
                },
            });
        }

        Ok(issues)
    }

    pub fn try_set_assignees<T: FmtDisplay>(&self, assignees: impl AsRef<[T]>) -> GitHubResult<(), IssueError> {
        let repository = self.get_parent();

        let assignees: Vec<String> = assignees.as_ref()
            .iter().map(|assignee| assignee.to_string())
            .collect();

        let ref payload = serde_json::json!({
            "assignees": assignees.as_slice(),
        });

        self.get_client()
            .post(format!("repos/{repository}/issues/{self}/assignees"))?
            .json(payload).send()?;

        Ok(())
    }

    pub fn try_get_assignees(&self) -> GitHubResult<Vec<User>, IssueError> {
        let repository = self.get_parent();

        let ref payload = serde_json::json!({
            "assignees": [],
        });

        let response = self.get_client()
            .post(format!("repos/{repository}/issues/{self}/assignees"))?
            .json(payload).send()?;

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            assignees: Vec<User>,
        }

        let Capsule { assignees } = response.json()?;

        Ok(assignees)
    }

    pub fn try_get_comment(&self, number: Number) -> GitHubResult<HandleIssueComment, IssueError>
   {
        Ok(HandleIssueComment::try_fetch(self, number)?)
    }

    pub fn try_has_comment(&self, number: Number) -> GitHubResult<bool, IssueError> {
        match HandleIssueComment::try_fetch(self, number) {
            Err(IssueCommentError::Nothing { .. }) => Ok(false),
            Err(error) => Err(IssueError::Comment(error)),
            Ok(_) => Ok(true),
        }
    }

    pub fn try_get_all_issue_comments(&self) -> GitHubResult<Vec<HandleIssueComment>, IssueError> {
        Ok(HandleIssueComment::try_fetch_all(self)?)
    }

    pub fn try_has_comments(&self) -> GitHubResult<bool, IssueError> {
        match HandleIssueComment::try_fetch_all(self) {
            Err(IssueCommentError::Nothing { .. }) => Ok(false),
            Err(error) => Err(IssueError::Comment(error)),
            Ok(_) => Ok(true),
        }
    }

    pub fn try_create_comment(&self, content: impl AsRef<str>) -> GitHubResult<HandleIssueComment, IssueError> {
        Ok(HandleIssueComment::try_create(self, content.as_ref())?)
    }

    pub fn try_delete_comment(&self, number: impl Into<Number>) -> GitHubResult<(), IssueError> {
        Ok(HandleIssueComment::try_delete(self, number)?)
    }
}

impl Into<Number> for &HandleIssue {
    fn into(self) -> Number {
        self.number.clone()
    }
}

impl Into<Number> for HandleIssue {
    fn into(self) -> Number {
        self.number.clone()
    }
}

impl<'a> GitHubProperties<'a> for HandleIssue {
    type Content = Issue;
    type Parent = HandleRepository;

    fn get_client(&'a self) -> &'a Client {
        self.get_parent()
            .get_client()
    }
    
    fn get_parent(&'a self) -> &'a Self::Parent {
        &(self.repository)
    }

    fn get_endpoint(&'a self) -> Cow<'a, str> {
        let Self { repository, .. } = { self };
        format!("repos/{repository}/issues/{self}").into()
    }
}

impl FmtDisplay for HandleIssue {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        write!(fmt, "{number}", number = {
            self.number.clone()
        })
    }
}
