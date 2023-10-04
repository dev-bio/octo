use std::fmt::{
    
    Formatter as FmtFormatter,
    Display as FmtDisplay,
    Result as FmtResult,
};

use serde::{
    
    Deserialize,
    Serialize,
};

#[derive(Default, Hash, Clone, Debug)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(Serialize, Deserialize)]
pub struct Sha(String);

impl AsRef<str> for Sha {
    fn as_ref(&self) -> &str {
        let Sha(value) = self;
        value.as_ref()
    }
}

impl From<String> for Sha {
    fn from(value: String) -> Self {
        Sha(value)
    }
}

impl From<&str> for Sha {
    fn from(value: &str) -> Self {
        Sha(value.to_string())
    }
}

impl FmtDisplay for Sha {
    fn fmt(&self, fmt: &mut FmtFormatter<'_>) -> FmtResult {
        write!(fmt, "{hash}", hash = self.as_ref())
    }
}