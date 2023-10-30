use std::{

    borrow::{Cow},

    fmt::{
        
        Formatter as FmtFormatter,
        Display as FmtDisplay,
        Result as FmtResult,
    }, 
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
    GitHubResult, 
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
    pub(crate) fn try_fetch(issue: &HandleIssue, number: impl Into<Number>) -> GitHubResult<HandleIssueComment, IssueCommentError> {
        let Comment { number, .. } = {

            let repository = issue.get_parent();
            let number = number.into();
            
            let result = {

                repository.get_client()
                    .get(format!("repos/{repository}/issues/comments/{number}"))?
                    .send()
            };

            match result {
                Err(ClientError::Response(ClientResponseError::Nothing { .. })) => {
                    return Err(IssueCommentError::Nothing { number })
                },
                Err(error) => return Err(error.into()),
                Ok(response) => response.json()?,
            }
        };

        Ok(HandleIssueComment {
            issue: issue.clone(),
            number,
        })
    }

    pub(crate) fn try_fetch_all(issue: &HandleIssue) -> GitHubResult<Vec<HandleIssueComment>, IssueCommentError> {
        let repository = issue.get_parent();

        let mut collection = Vec::new();
        let mut page = 0;

        loop {

            page = { page + 1 };

            let capsules: Vec<Comment> = {
                let ref query = [
                    ("per_page", 100),
                    ("page", page),
                ];

                let result = {
                    
                    repository.get_client()
                        .get(format!("repos/{repository}/issues{issue}/comments"))?
                        .query(query)
                        .send()
                };

                match result {
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
        for Comment { number, .. } in collection {
            issues.push(HandleIssueComment {
                issue: issue.clone(), number
            });
        }

        Ok(issues)
    }

    pub(crate) fn try_create(issue: &HandleIssue, content: impl AsRef<str>) -> GitHubResult<HandleIssueComment, IssueCommentError> {
        let repository = issue.get_parent();

        let ref payload = serde_json::json!({
            "body": content.as_ref()
                .to_string()
        });

        let Comment { number, .. } = {

            repository.get_client()
                .post(format!("repos/{repository}/issues/{issue}/comments"))?
                .json(payload)
                .send()?
                .json()?
        };

        Ok(HandleIssueComment {
            issue: issue.clone(),
            number,
        })
    }

    pub(crate) fn try_delete(issue: &HandleIssue, number: impl Into<Number>) -> GitHubResult<(), IssueCommentError> {
        let repository = issue.get_parent();
        let number = number.into();
        
        let _ = {

            repository.get_client()
                .delete(format!("repos/{repository}/issues/comments/{number}"))?
                .send()?
        };

        Ok(())
    }
}

impl<'a> GitHubProperties<'a> for HandleIssueComment {
    type Content = Comment;
    type Parent = HandleIssue;
    
    fn get_client(&'a self) -> &'a Client {
        self.get_parent()
            .get_client()
    }
    
    fn get_parent(&'a self) -> &'a Self::Parent {
        &(self.issue)
    }

    fn get_endpoint(&'a self) -> Cow<'a, str> {
        format!("repos/{repository}/issues/comments/{self}", repository = {
            self.issue.get_parent()
        }).into()
    }
}

impl Into<Number> for &HandleIssueComment {
    fn into(self) -> Number {
        self.number.clone()
    }
}

impl Into<Number> for HandleIssueComment {
    fn into(self) -> Number {
        self.number.clone()
    }
}

impl FmtDisplay for HandleIssueComment {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        write!(fmt, "{number}", number = {
            self.number.clone()
        })
    }
}
