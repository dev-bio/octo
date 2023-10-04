use serde::{
    
    Deserialize,
    Serialize, 
};

use crate::models::common::{

    issue::{Issue, IssueState},
    user::{User},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum EventIssue {
    #[serde(rename = "opened")]
    Opened { issue: Issue },
    #[serde(rename = "closed")]
    Closed { issue: Issue },
    #[serde(rename = "edited")]
    Edited { issue: Issue },
    #[serde(rename = "reopened")]
    Reopened { issue: Issue },
    #[serde(rename = "deleted")]
    Deleted { issue: Issue },
}

impl EventIssue {
    pub fn get_state(&self) -> IssueState {
        match self {
            EventIssue::Reopened { issue, .. } |
            EventIssue::Deleted { issue, .. } |
            EventIssue::Edited { issue, .. } |
            EventIssue::Opened { issue, .. } |
            EventIssue::Closed { issue, .. } => issue.get_state(),
        }
    }

    pub fn get_author(&self) -> User {
        match self {
            EventIssue::Reopened { issue, .. } |
            EventIssue::Deleted { issue, .. } |
            EventIssue::Edited { issue, .. } |
            EventIssue::Opened { issue, .. } |
            EventIssue::Closed { issue, .. } => issue.get_author(),
        }
    }

    pub fn get_body(&self) -> String {
        match self {
            EventIssue::Reopened { issue, .. } |
            EventIssue::Deleted { issue, .. } |
            EventIssue::Edited { issue, .. } |
            EventIssue::Opened { issue, .. } |
            EventIssue::Closed { issue, .. } => issue.get_body(),
        }
    }

    pub fn get_number(&self) -> usize {
        match self {
            EventIssue::Reopened { issue, .. } |
            EventIssue::Deleted { issue, .. } |
            EventIssue::Edited { issue, .. } |
            EventIssue::Opened { issue, .. } |
            EventIssue::Closed { issue, .. } => issue.get_number(),
        }
    }
}