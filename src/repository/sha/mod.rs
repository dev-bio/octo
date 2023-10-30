use std::{

    borrow::{Cow},
    ops::{Deref}, 

    fmt::{
        
        Formatter as FmtFormatter,
        Display as FmtDisplay,
        Result as FmtResult,
    }, 
    
};

use serde::{
    
    Deserialize,
    Serialize,
};

#[derive(Default, Hash, Clone, Debug)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(Serialize, Deserialize)]
pub struct Sha<'h>(Cow<'h, str>);

impl<'h> Sha<'h> {
    pub fn to_owned(&self) -> Sha<'static> {
        Sha(Cow::Owned(self.as_ref()
            .to_owned()))
    }
}

impl<'h> AsRef<str> for Sha<'h> {
    fn as_ref(&self) -> &str {
        let Sha(value) = { self };
        value.as_ref()
    }
}

impl<'h> Deref for Sha<'h> {
    type Target = Cow<'h, str>;

    fn deref(&self) -> &Self::Target {
        let Sha(value) = { self };
        value
    }
}

impl<'h> From<String> for Sha<'h> {
    fn from(value: String) -> Self {
        Sha(Cow::Owned(value))
    }
}

impl<'h> From<&'h str> for Sha<'h> {
    fn from(value: &'h str) -> Self {
        Sha(Cow::Borrowed(value))
    }
}

impl<'h> FmtDisplay for Sha<'h> {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        write!(fmt, "{hash}", hash = self.as_ref())
    }
}