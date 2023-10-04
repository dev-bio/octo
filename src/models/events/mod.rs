use serde::{
    
    Deserialize,
    Serialize, 
};

pub mod payloads;
pub use payloads::{
    
    EventIssueComment,
    EventIssue, 
};

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[serde(tag = "event_name", content = "event")]
pub enum Event {
    #[serde(rename = "issue_comment")]
    IssueComment(EventIssueComment),
    #[serde(rename = "issues")]
    Issue(EventIssue),
    #[serde(rename = "schedule")]
    Schedule,
}