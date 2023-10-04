use std::fmt::{
    
    Formatter as FmtFormatter,
    Display as FmtDisplay,
    Result as FmtResult,
};

use crate::{

    repository::issue::{HandleIssue},

    client::{

        ClientResponseError,
        ClientError,
        Client,
    },

    models::common::issue::comment::{Comment},
    
    GitHubProperties,
    GitHubEndpoint,
    GitHubResult, 
    GitHubObject,
    Number,
};

use thiserror::{Error};

#[derive(Error, Debug)]
pub enum IssueCommentError {
    #[error("Client error!")]
    Client(#[from] ClientError),
    #[error("Failed to fetch issue comment author: '{author}'")]
    Author { author: String },
    #[error("Issue comment not found: {number}")]
    Nothing { number: Number },
}

#[derive(Clone, Debug)]
pub struct HandleIssueComment {
    issue: HandleIssue,
    number: Number,
}

impl HandleIssueComment {
    pub(crate) fn try_fetch(issue: HandleIssue, number: Number) -> GitHubResult<HandleIssueComment, IssueCommentError> {
        let repository = issue.get_parent();
        let client = issue.get_client();

        let Comment { number, .. } = {
            let request = client.get(format!("repos/{repository}/issues/comments/{number}"))?;
            match request.send() {
                Err(ClientError::Response(ClientResponseError::Nothing { .. })) => {
                    return Err(IssueCommentError::Nothing { number })
                },
                Err(error) => return Err(error.into()),
                Ok(response) => response.json()?,
            }
        };

        Ok(HandleIssueComment {

            issue,
            number,
        })
    }

    pub(crate) fn try_fetch_all(issue: HandleIssue) -> GitHubResult<Vec<HandleIssueComment>, IssueCommentError> {
        let repository = issue.get_parent();
        let client = issue.get_client();

        let mut collection = Vec::new();
        let mut page = 0;

        loop {

            page = { page + 1 };

            let capsules: Vec<Comment> = {
                let ref query = [
                    ("per_page", 100),
                    ("page", page),
                ];

                let request = client.get(format!("repos/{repository}/issues{issue}/comments"))?
                    .query(query);

                match request.send() {
                    Err(ClientError::Response(ClientResponseError::Nothing { .. })) => break,
                    Err(error) => return Err(error.into()),
                    Ok(response) => response.json()?,
                }
            };

            collection.extend_from_slice({
                capsules.as_slice()
            });

            if capsules.len() < 100 {
                break
            }
        }

        let mut issues = Vec::new();
        for Comment { number, .. } in collection.iter() {
            issues.push(HandleIssueComment { 
                issue: issue.clone(),
                number: number.clone(), 
            });
        }

        Ok(issues)
    }

    pub fn try_create(issue: HandleIssue, content: impl AsRef<str>) -> GitHubResult<HandleIssueComment, IssueCommentError> {
        let repository = issue.get_parent();
        let client = issue.get_client();

        let ref payload = serde_json::json!({
            "body": content.as_ref()
                .to_string()
        });

        let Comment { number, .. } = client.post(format!("repos/{repository}/issues/{issue}/comments"))?
            .json(payload).send()?.json()?;

        Ok(HandleIssueComment {
            issue, number
        })
    }

    pub fn try_delete(issue: HandleIssue, number: usize) -> GitHubResult<(), IssueCommentError> {
        let repository = issue.get_parent();
        let client = repository.get_client();

        client.delete(format!("repos/{repository}/issues/comments/{number}"))?
            .send()?;

        Ok(())
    }

    pub fn get_issue(&self) -> HandleIssue {
        self.issue.clone()
    }
}

impl GitHubObject for HandleIssueComment {
    fn get_number(&self) -> Number {
        self.number.clone()
    }
}

impl GitHubEndpoint for HandleIssueComment {
    fn get_client(&self) -> Client {
        self.issue.get_client()
    }

    fn get_endpoint(&self) -> String {
        format!("repos/{repository}/issues/comments/{self}", repository = {
            self.issue.get_parent()
        })
    }
}

impl GitHubProperties for HandleIssueComment {
    type Content = Comment;
    type Parent = HandleIssue;

    fn get_parent(&self) -> Self::Parent {
        self.issue.clone()
    }
}

impl FmtDisplay for HandleIssueComment {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        write!(fmt, "{number}", number = {
            self.number.clone()
        })
    }
}

impl Into<Number> for HandleIssueComment {
    fn into(self) -> Number {
        self.number.clone()
    }
}
