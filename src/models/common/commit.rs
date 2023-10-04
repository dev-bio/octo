use serde::{
    
    Deserialize,
    Serialize, Serializer, Deserializer,
};

use crate::{common::{Date}, repository::sha::Sha};

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct CommitAuthor {
    pub email: String,
    pub name: String,
    pub date: Date,
}

impl CommitAuthor {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_email(&self) -> String {
        self.email.clone()
    }

    pub fn get_date(&self) -> Date {
        self.date.clone()
    }
}

#[derive(Debug, Clone)]
pub enum CommitVerification {
    Signed {
        signature: String,
        payload: String,
    },
    None,
}

impl CommitVerification {
    pub fn is_verified(&self) -> bool {
        match self {
            CommitVerification::Signed { .. } => true,
            CommitVerification::None => false,
        }
    }
}

impl Serialize for CommitVerification {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {

        use serde::ser::{SerializeStruct};

        match self {
            CommitVerification::Signed { signature, payload } => {
                let mut state = serializer.serialize_struct("CommitVerification", 3)?;
                let ref verified = true;
                
                state.serialize_field("verified", verified)?;
                state.serialize_field("signature", signature)?;
                state.serialize_field("payload", payload)?;
                
                state.end()
            },
            CommitVerification::None => {
                let mut state = serializer.serialize_struct("CommitVerification", 1)?;
                let ref verified = false;

                state.serialize_field("verified", verified)?;

                state.end()
            },
        }
    }
}

impl<'de> Deserialize<'de> for CommitVerification {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        #[derive(Deserialize)]
        struct Capsule {
            verified: bool,
            signature: Option<String>,
            payload: Option<String>,
        }

        let Capsule { verified, signature, payload } = {
            Capsule::deserialize(deserializer)?
        };

        if verified {

            use serde::de::{Error};

            Ok(CommitVerification::Signed {
                signature: signature.ok_or(Error::missing_field("signature"))?,
                payload: payload.ok_or(Error::missing_field("payload"))?,
            })
        }
        
        else {

            Ok(CommitVerification::None)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Commit {
    pub author: CommitAuthor,
    pub verified: CommitVerification,
    pub parents: Vec<Sha>,
}

impl<'de> Deserialize<'de> for Commit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        #[derive(Deserialize)]
        struct CapsuleParent {
            sha: Sha,
        }

        #[derive(Deserialize)]
        struct CapsuleCommit {
            author: CommitAuthor,
            verified: CommitVerification,
        }

        #[derive(Deserialize)]
        struct Capsule {
            commit: CapsuleCommit,
            parents: Vec<CapsuleParent>,
        }

        let Capsule { commit: CapsuleCommit { author, verified }, parents } = {
            Capsule::deserialize(deserializer)?
        };

        Ok(Commit {

            verified, 
            author, 
            parents: parents.into_iter()
                .map(|CapsuleParent { sha }| { sha })
                .collect(),
        })
    }
}