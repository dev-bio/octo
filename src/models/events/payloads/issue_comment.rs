use serde::{
    
    Deserialize,
    Serialize, 
};

use crate::models::{

    common::{

        issue::comment::{Comment},
        issue::{Issue},
    },
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum EventIssueComment {
    #[serde(rename = "created")]
    Created { issue: Issue, comment: Comment },
    #[serde(rename = "edited")]
    Edited { issue: Issue, comment: Comment },
    #[serde(rename = "deleted")]
    Deleted { issue: Issue, comment: Comment },
}

impl EventIssueComment {
    pub fn get_issue_number(&self) -> usize {
        match self {
            EventIssueComment::Deleted { issue, .. } |
            EventIssueComment::Created { issue, .. } |
            EventIssueComment::Edited { issue, .. } => issue.get_number(),
        }
    }

    pub fn get_issue_comment_number(&self) -> usize {
        match self {
            EventIssueComment::Deleted { comment, .. } |
            EventIssueComment::Created { comment, .. } |
            EventIssueComment::Edited { comment, .. } => comment.get_number(),
        }
    }
}
