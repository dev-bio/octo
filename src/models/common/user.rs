use std::fmt::{

    Formatter as FmtFormatter,
    Display as FmtDisplay,
    Result as FmtResult,
    Debug as FmtDebug,
};


use serde::{
    
    Deserialize,
    Serialize, 
};

use crate::Number;

#[derive(Clone, Hash, Eq)]
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum User {
    #[serde(rename = "Organization")]
    Organization {
        #[serde(rename = "login")]
        name: String,
        #[serde(rename = "id")]
        number: usize,
    },
    #[serde(rename = "Mannequin")]
    Mannequin {
        #[serde(rename = "login")]
        name: String,
        #[serde(rename = "id")]
        number: usize,
    },
    #[serde(rename = "User")]
    User {
        #[serde(rename = "login")]
        name: String,
        #[serde(rename = "id")]
        number: usize,
    },
    #[serde(rename = "Bot")]
    Bot {
        #[serde(rename = "login")]
        name: String,
        #[serde(rename = "id")]
        number: usize,
    },
}

impl User {
    pub fn get_name(&self) -> String {
        match self {
            User::Organization { name, .. } |
            User::Mannequin { name, .. } |
            User::User { name, .. } |
            User::Bot { name, .. } => name.clone(),
        }
    }

    pub fn get_number(&self) -> Number {
        match self {
            User::Organization { number, .. } |
            User::Mannequin { number, .. } |
            User::User { number, .. } |
            User::Bot { number, .. } => number.clone(),
        }
    }

    pub fn is_organization(&self) -> bool {
        match self {
            User::Organization { .. } => true,
            _ => false,
        }
    }

    pub fn is_mannequin(&self) -> bool {
        match self {
            User::Mannequin { .. } => true,
            _ => false,
        }
    }

    pub fn is_user(&self) -> bool {
        match self {
            User::User { .. } => true,
            _ => false,
        }
    }

    pub fn is_bot(&self) -> bool {
        match self {
            User::Bot { .. } => true,
            _ => false,
        }
    }
}

impl PartialEq for User {
    fn eq(&self, other: &User) -> bool {
        let ref one = self.get_number();
        let ref two = other.get_number();

        one.eq(two)
    }
}

impl FmtDisplay for User {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        match self {
            User::Organization { name, .. } |
            User::Mannequin { name, .. } |
            User::User { name, .. } |
            User::Bot { name, .. } => write!(fmt, "{name}"),
        }
    }
}

impl FmtDebug for User {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        match self {
            User::Organization { .. } => write!(fmt, "organization"),
            User::Mannequin { .. } => write!(fmt, "mannequin"),
            User::User { .. } => write!(fmt, "user"),
            User::Bot { .. } => write!(fmt, "bot"),
        }
    }
}

impl AsRef<str> for User {
    fn as_ref(&self) -> &str {
        match self {
            User::Organization { name, .. } |
            User::Mannequin { name, .. } |
            User::User { name, .. } |
            User::Bot { name, .. } => name.as_ref(),
        }
    }
}