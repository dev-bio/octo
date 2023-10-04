use std::fmt::{

    Formatter as FmtFormatter,
    Display as FmtDisplay,
    Result as FmtResult,
};

use serde::{
    
    Deserialize,
    Serialize, 
};

use crate::{

    models::common::user::{User},

    Number,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub struct Comment {
    pub(crate) body: String,
    #[serde(rename = "user")]
    pub(crate) author: User,
    #[serde(rename = "id")]
    pub(crate) number: usize,
}

impl Comment {
    pub fn get_author(&self) -> User {
        self.author.clone()
    }

    pub fn get_number(&self) -> Number {
        self.number
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
}

impl FmtDisplay for Comment {
    fn fmt(&self, fmt: &mut FmtFormatter) -> FmtResult {
        write!(fmt, "{number}", number = {
            self.number.clone()
        })
    }
}

impl Into<Number> for Comment {
    fn into(self) -> Number {
        self.number.clone()
    }
}