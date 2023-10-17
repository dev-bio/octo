use std::{

    sync::{Arc, Weak}, 
    
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
    reference: Weak<HandleIssueComment>,
    issue: Arc<HandleIssue>,
    number: Number,
}

impl HandleIssueComment {
    pub(crate) fn try_fetch(issue: impl Into<Arc<HandleIssue>>, number: Number) -> GitHubResult<Arc<HandleIssueComment>, IssueCommentError> {
        let issue = issue.into();

        
        let Comment { number, .. } = {

            let repository = issue.get_parent();
            
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

        Ok(Arc::new_cyclic(|reference| HandleIssueComment {
            reference: reference.clone(), issue, number
        }))
    }

    pub(crate) fn try_fetch_all(issue: impl Into<Arc<HandleIssue>>) -> GitHubResult<Vec<Arc<HandleIssueComment>>, IssueCommentError> {
        let issue = issue.into();

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
            issues.push(Arc::new_cyclic(|reference| HandleIssueComment {
                reference: reference.clone(), issue: issue.clone(), number
            }));
        }

        Ok(issues)
    }

    pub fn try_create(issue: impl Into<Arc<HandleIssue>>, content: impl AsRef<str>) -> GitHubResult<Arc<HandleIssueComment>, IssueCommentError> {
        let issue = issue.into();

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

        Ok(Arc::new_cyclic(|reference| HandleIssueComment {
            reference: reference.clone(), issue, number
        }))
    }

    pub fn try_delete(issue: impl AsRef<HandleIssue>, number: usize) -> GitHubResult<(), IssueCommentError> {
        let repository = issue.as_ref()
            .get_parent();
        
        let _ = {

            repository.get_client()
                .delete(format!("repos/{repository}/issues/comments/{number}"))?
                .send()?
        };

        Ok(())
    }
}

impl GitHubProperties for HandleIssueComment {
    type Content = Comment;
    type Parent = Arc<HandleIssue>;
    
    fn get_client(&self) -> Client {
        self.get_parent()
            .get_client()
    }
    
    fn get_parent(&self) -> Self::Parent {
        self.issue.clone()
    }

    fn get_endpoint(&self) -> String {
        format!("repos/{repository}/issues/comments/{self}", repository = {
            self.issue.get_parent()
        })
    }

    fn get_reference(&self) -> Arc<Self> {
        self.reference.upgrade()
            .unwrap()
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
