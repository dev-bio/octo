use std::{

    fmt::{

        Formatter as FmtFormatter,
        Display as FmtDisplay,
        Result as FmtResult,
    },

    ops::{

        DerefMut,
        Deref,
    },
};


use serde::{
    
    Deserializer,
    Deserialize,
    Serialize, 
};

use crate::{Number};

use super::user::{User};

pub mod comment;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub enum IssueState {
    #[serde(rename = "closed")] Closed,
    #[serde(rename = "open")] Open,
}

impl IssueState {
    pub fn is_open(&self) -> bool {
        match self {
            IssueState::Open => true,
            IssueState::Closed => false,
        }
    }

    pub fn is_closed(&self) -> bool {
        match self {
            IssueState::Open => false,
            IssueState::Closed => true,
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[derive(Serialize)]
#[serde(untagged)]
pub enum Issue {
    PullRequest(IssueContent),
    Plain(IssueContent),
}

impl Issue {
    pub fn get_author(&self) -> User {
        match self {
            Issue::Plain(content) => content.get_author(),
            Issue::PullRequest(content) => content.get_author(),
        }
    }
    
    pub fn is_pull_request(&self) -> bool {
        match self {
            Issue::Plain(_) => false,
            Issue::PullRequest(_) => true,
        }
    }

    pub fn is_plain(&self) -> bool {
        match self {
            Issue::Plain(_) => true,
            Issue::PullRequest(_) => false,
        }
    }

    pub fn get_state(&self) -> IssueState {
        match self {
            Issue::Plain(content) => content.get_state(),
            Issue::PullRequest(content) => content.get_state(),
        }
    }

    pub fn is_open(&self) -> bool {
        match self {
            Issue::Plain(content) => content.is_open(),
            Issue::PullRequest(content) => content.is_open(),
        }
    }

    pub fn is_closed(&self) -> bool {
        match self {
            Issue::Plain(content) => content.is_closed(),
            Issue::PullRequest(content) => content.is_closed(),
        }
    }
}

impl<'de> Deserialize<'de> for Issue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct CapsulePullRequest {

            // ..
        }

        #[derive(Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            pull_request: Option<CapsulePullRequest>,
            #[serde(flatten)]
            issue: IssueContent,
        }

        let data = Capsule::deserialize(deserializer)?;

        Ok(match data.pull_request.is_some() {
            true => Issue::PullRequest(data.issue),
            _ => Issue::Plain(data.issue),
        })
    }
}

impl FmtDisplay for Issue {
    fn fmt(&self, fmt: &mut FmtFormatter) -> FmtResult {
        match self {
            Issue::PullRequest(content) => content.fmt(fmt),
            Issue::Plain(content) => content.fmt(fmt),
        }
    }
}

impl Deref for Issue {
    type Target = IssueContent;

    fn deref(&self) -> &Self::Target {
        match self {
            Issue::Plain(content) => content,
            Issue::PullRequest(content) => content,
        }
    }
}

impl DerefMut for Issue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Issue::Plain(content) => content,
            Issue::PullRequest(content) => content,
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub struct IssueContent {
    pub(crate) assignees: Option<Vec<User>>,
    pub(crate) number: Number,
    #[serde(rename = "user")]
    pub(crate) author: User,
    pub(crate) title: String,
    pub(crate) body: String,
    pub(crate) state: IssueState 
}

impl IssueContent {
    pub fn get_author(&self) -> User {
        self.author.clone()
    }

    pub fn get_number(&self) -> usize {
        self.number
    }

    pub fn get_title(&self) -> String {
        self.title.clone()
    }

    pub fn set_title(&mut self, title: impl AsRef<str>) {
        self.title = title.as_ref()
            .to_owned();
    }

    pub fn with_title(mut self, title: impl AsRef<str>) -> Self {
        self.set_title(title);
        self
    }

    pub fn get_body(&self) -> String {
        self.body.clone()
    }

    pub fn set_body(&mut self, body: impl AsRef<str>) {
        self.body = body.as_ref()
            .to_owned();
    }

    pub fn with_body(mut self, body: impl AsRef<str>) -> Self {
        self.set_body(body);
        self
    }

    pub fn get_state(&self) -> IssueState {
        self.state.clone()
    }

    pub fn set_state(&mut self, state: IssueState) {
        self.state = state;
    }

    pub fn with_state(mut self, state: IssueState) -> Self {
        self.set_state(state);
        self
    }

    pub fn get_assignees(&self) -> Vec<User> {
        self.assignees.clone()
            .unwrap_or_default()
    }

    pub fn set_assignees(&mut self, assignees: impl AsRef<[User]>) {
        self.assignees = Some({
            assignees.as_ref()
                .to_owned()
        });
    }

    pub fn with_assignees(mut self, assignees: impl AsRef<[User]>) -> Self {
        self.set_assignees(assignees);
        self
    }

    pub fn is_closed(&self) -> bool {
        match self.state {
            IssueState::Closed => true,
            _ => false,
        }
    }

    pub fn close(&mut self) {
        self.state = IssueState::Closed;
    }

    pub fn as_closed(mut self) -> Self {
        self.close();
        self
    }

    pub fn is_open(&self) -> bool {
        match self.state {
            IssueState::Open => true,
            _ => false,
        }
    }

    pub fn open(&mut self) {
        self.state = IssueState::Open;
    }

    pub fn as_open(mut self) -> Self {
        self.open();
        self
    }
}

impl FmtDisplay for IssueContent {
    fn fmt(&self, fmt: &mut FmtFormatter) -> FmtResult {
        write!(fmt, "{number}", number = {
            self.number.clone()
        })
    }
}

impl Into<Number> for IssueContent {
    fn into(self) -> Number {
        self.number.clone()
    }
}

#[cfg(test)]
mod tests {

    use super::{Issue};

    #[test]
    fn test_serialize() {
        let plain = include_str!("test_data/plain.json");

        let ref issue: Issue = serde_json::from_str(plain)
            .unwrap();

        assert_eq!(issue.is_plain(), true);

        let pretty = serde_json::to_string_pretty(issue)
            .unwrap();

        assert_eq!(plain, pretty.as_str());

        let pull = include_str!("test_data/pull.json");

        let ref issue: Issue = serde_json::from_str(pull)
            .unwrap();

        assert_eq!(issue.is_pull_request(), true);

        let pretty = serde_json::to_string_pretty(issue)
            .unwrap();

        assert_eq!(plain, pretty.as_str());
    }

    #[test]
    fn test_deserialize() {
        let raw = include_str!("test_data/pull.json");

        let ref issue: Issue = serde_json::from_str(raw)
            .unwrap();

        assert_eq!(issue.is_pull_request(), true);

        let raw = include_str!("test_data/plain.json");

        let ref issue: Issue = serde_json::from_str(raw)
            .unwrap();

        assert_eq!(issue.is_plain(), true);
    }
}