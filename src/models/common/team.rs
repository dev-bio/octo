use std::fmt::{

    Formatter as FmtFormatter,
    Display as FmtDisplay,
    Result as FmtResult,
};


use serde::{
    
    Deserialize,
    Serialize, 
};

use crate::{Number};

#[derive(Clone, Debug, Eq)]
#[derive(Serialize, Deserialize)]
pub struct Team {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) slug: String,
    #[serde(rename = "id")]
    pub(crate) number: Number,
}

impl Team {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_description(&self) -> String {
        self.description.clone()
    }

    pub fn get_slug(&self) -> String {
        self.slug.clone()
    }

    pub fn get_number(&self) -> Number {
        self.number.clone()
    }
}

impl PartialEq for Team {
    fn eq(&self, other: &Team) -> bool {
        let ref one = self.get_number();
        let ref two = other.get_number();

        one.eq(two) && {
            let ref one = self.get_slug();
            let ref two = other.get_slug();

            one.eq(two)
        }
    }
}

impl FmtDisplay for Team {
    fn fmt(&self, fmt: &mut FmtFormatter) -> FmtResult {
        write!(fmt, "{slug}", slug = {
            self.slug.clone()
        })
    }
}

impl AsRef<str> for Team {
    fn as_ref(&self) -> &str {
        self.slug.as_ref()
    }
}

impl Into<Number> for Team {
    fn into(self) -> Number {
        self.number.clone()
    }
}