use crate::common::{Date};

use serde::{
    
    Deserialize,
    Serialize,
};

use crate::models::common::user::{User};

use std::{

    fmt::{
    
        Formatter as FmtFormatter,
        Display as FmtDisplay,
        Result as FmtResult,
    },
};

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub enum Visibility {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "internal")]
    Internal,
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub enum Status {
    #[serde(rename = "enabled")]
    Enabled,
    #[serde(rename = "disabled")]
    Disabled,
}

impl Default for Status {
    fn default() -> Status {
        Status::Disabled
    }
}

impl From<bool> for Status {
    fn from(status: bool) -> Status {
        if status { Status::Enabled } else {
            Status::Disabled
        }
    }
}

#[derive(Default, Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct Security {
    advanced_security: Status,
    secret_scanning_push_protection: Status,
    secret_scanning: Status,
}

impl Security {
    pub fn new() -> Security {
        Security {
            advanced_security: Status::Disabled,
            secret_scanning_push_protection: Status::Disabled,
            secret_scanning: Status::Disabled,
        }
    }

    pub fn with_advanced_security(mut self, status: bool) -> Security {
        self.advanced_security = status.into();
        self
    }

    pub fn with_secret_scanning_push_protection(mut self, status: bool) -> Security {
        self.secret_scanning_push_protection = status.into();
        self
    }

    pub fn with_secret_scanning(mut self, status: bool) -> Security {
        self.secret_scanning = status.into();
        self
   }
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct Repository {
    name: String,
    description: String,
    homepage: String,

    #[serde(skip_serializing)]
    owner: User,

    default_branch: String,
    visibility: Visibility,

    #[serde(rename = "is_template")]
    template: bool,

    private: bool,

    #[serde(rename = "has_issues")]
    issues: bool,

    #[serde(rename = "has_projects")]
    projects: bool,

    #[serde(rename = "has_wiki")]
    wiki: bool,

    #[serde(rename = "has_downloads")]
    downloads: bool,

    #[serde(rename = "security_and_analysis")]
    #[serde(skip_serializing_if = "Option::is_none")]
    security: Option<Security>,

    #[serde(rename = "allow_forking")]
    forking: bool,

    #[serde(rename = "web_commit_signoff_required")]
    signoff: bool,

    archived: bool,

    #[serde(rename = "created_at")]
    #[serde(skip_serializing)]
    date_created: Option<Date>,

    #[serde(rename = "updated_at")]
    #[serde(skip_serializing)]
    date_updated: Option<Date>,

    #[serde(rename = "pushed_at")]
    #[serde(skip_serializing)]
    date_pushed: Option<Date>,
}

impl Repository {

    pub fn with_name(mut self, name: String) -> Repository {
        self.name = name;
        self
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn with_description(mut self, description: String) -> Repository {
        self.description = description;
        self
    }

    pub fn get_description(&self) -> String {
        self.description.clone()
    }

    pub fn with_homepage(mut self, homepage: String) -> Repository {
        self.homepage = homepage;
        self
    }

    pub fn get_homepage(&self) -> String {
        self.homepage.clone()
    }

    pub fn get_owner(&self) -> User {
        self.owner.clone()
    }

    pub fn with_default_branch(mut self, default_branch: String) -> Repository {
        self.default_branch = default_branch;
        self
    }

    pub fn get_default_branch(&self) -> String {
        self.default_branch.clone()
    }

    pub fn set_default_branch(&mut self, default_branch: String) {
        self.default_branch = default_branch;
    }

    pub fn with_visibility(mut self, visibility: Visibility) -> Repository {
        self.visibility = visibility;
        self
    }

    pub fn get_visibility(&self) -> Visibility {
        self.visibility.clone()
    }

    pub fn set_visibility(&mut self, visibility: Visibility) {
        self.visibility = visibility;
    }

    pub fn with_template(mut self, template: bool) -> Repository {
        self.template = template;
        self
    }

    pub fn has_template(&self) -> bool {
        self.template.clone()
    }

    pub fn set_template(&mut self, template: bool) {
        self.template = template;
    }

    pub fn with_private(mut self, private: bool) -> Repository {
        self.private = private;
        self
    }

    pub fn has_private(&self) -> bool {
        self.private.clone()
    }

    pub fn set_private(&mut self, private: bool) {
        self.private = private;
    }

    pub fn with_issues(mut self, issues: bool) -> Repository {
        self.issues = issues;
        self
    }

    pub fn has_issues(&self) -> bool {
        self.issues.clone()
    }

    pub fn set_issues(&mut self, issues: bool) {
        self.issues = issues;
    }

    pub fn with_projects(mut self, projects: bool) -> Repository {
        self.projects = projects;
        self
    }

    pub fn has_projects(&self) -> bool {
        self.projects.clone()
    }

    pub fn set_projects(&mut self, projects: bool) {
        self.projects = projects;
    }

    pub fn with_wiki(mut self, wiki: bool) -> Repository {
        self.wiki = wiki;
        self
    }

    pub fn has_wiki(&self) -> bool {
        self.wiki.clone()
    }

    pub fn set_wiki(&mut self, wiki: bool) {
        self.wiki = wiki;
    }

    pub fn with_downloads(mut self, downloads: bool) -> Repository {
        self.downloads = downloads;
        self
    }

    pub fn has_downloads(&self) -> bool {
        self.downloads.clone()
    }

    pub fn set_downloads(&mut self, downloads: bool) {
        self.downloads = downloads;
    }

    pub fn with_security(mut self, security: Option<Security>) -> Repository {
        self.security = security;
        self
    }

    pub fn get_security(&self) -> Option<Security> {
        self.security.clone()
    }

    pub fn set_security(&mut self, security: Option<Security>) {
        self.security = security;
    }

    pub fn with_forking(mut self, forking: bool) -> Repository {
        self.forking = forking;
        self
    }

    pub fn has_forking(&self) -> bool {
        self.forking.clone()
    }

    pub fn set_forking(&mut self, forking: bool) {
        self.forking = forking;
    }

    pub fn with_signoff(mut self, signoff: bool) -> Repository {
        self.signoff = signoff;
        self
    }

    pub fn has_signoff(&self) -> bool {
        self.signoff.clone()
    }

    pub fn set_signoff(&mut self, signoff: bool) {
        self.signoff = signoff;
    }

    pub fn with_archived(mut self, archived: bool) -> Repository {
        self.archived = archived;
        self
    }

    pub fn has_archived(&self) -> bool {
        self.archived.clone()
    }

    pub fn set_archived(&mut self, archived: bool) {
        self.archived = archived;
    }

    pub fn get_date_created(&self) -> Option<Date> {
        self.date_created.clone()
    }

    pub fn get_date_updated(&self) -> Option<Date> {
        self.date_updated.clone()
    }

    pub fn get_date_pushed(&self) -> Option<Date> {
        self.date_pushed.clone()
    }
}

impl FmtDisplay for Repository {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        write!(fmt, "{name}", name = self.name)
    }
}