use serde::{

    de::{Error},

    Deserializer,
    Deserialize,

    Serializer,
    Serialize,
};

use thiserror::{Error};

use crate::{
    
    client::{ClientError},
    
    GitHubResult, 
};

use crate::{

    repository::{
        
        sha::{Sha},
        HandleRepository,
    },
};

#[derive(Error, Debug)]
pub enum BlobError {
    #[error("Client error!")]
    Client(#[from] ClientError),
}

#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
#[serde(tag = "encoding")]
pub enum Blob {

    #[serde(rename = "base64")]
    Binary {

        #[serde(deserialize_with = "deserialize")]
        #[serde(serialize_with = "serialize")]
        content: Vec<u8>,
        #[serde(skip_serializing)]
        sha: Sha,
    },

    #[serde(rename = "utf-8")]
    Text {

        content: String,
        #[serde(skip_serializing)]
        sha: Sha,
    },
}

impl Blob {
    pub fn try_fetch(repository: HandleRepository, sha: impl Into<Sha>) -> GitHubResult<Blob, BlobError> {
        let sha = sha.into();

        let blob = {

            repository.get_client()
                .get(format!("repos/{repository}/git/blobs/{sha}"))?
                .send()?
                .json()?
        };

        Ok(blob)
    }

    pub fn try_create_text_blob(repository: HandleRepository, text: impl AsRef<str>) -> GitHubResult<Blob, BlobError> {
        let text = text.as_ref();

        let ref blob = serde_json::json!({
            "encoding": "utf-8",
            "content": text,
        });
        
        let result = repository.get_client()
            .post(format!("repos/{repository}/git/blobs"))?
            .json(blob)
            .send()?;
        
        #[derive(Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            sha: Sha,
        }

        let Capsule { sha } = result.json()?;

        Ok(Blob::Text { content: text.to_owned(), sha })
    }

    pub fn try_create_binary_blob(repository: HandleRepository, binary: impl AsRef<[u8]>) -> GitHubResult<Blob, BlobError> {
        let binary = binary.as_ref();

        use base64::{

            engine::general_purpose::{STANDARD},
            Engine,
        };

        let ref blob = serde_json::json!({
            "encoding": "base64",
            "content": STANDARD.encode({
                binary.as_ref()
            }),
        });

        let result = repository.get_client()
            .post(format!("repos/{repository}/git/blobs"))?
            .json(blob)
            .send()?;
        
        #[derive(Debug)]
        #[derive(Deserialize)]
        struct Capsule {
            sha: Sha,
        }

        let Capsule { sha } = result.json()?;

        Ok(Blob::Binary { content: binary.to_owned(), sha })
    }

    pub fn get_sha(&self) -> Sha {
        match self {
            Blob::Binary { sha, .. } => sha.clone(),
            Blob::Text { sha, .. } => sha.clone(),
        }
    }
}

fn serialize<S>(value: impl AsRef<[u8]>, serializer: S) -> GitHubResult<S::Ok, S::Error>
where S: Serializer {

    use base64::{

        engine::general_purpose::{STANDARD},
        Engine,
    };

    serializer.serialize_some(STANDARD.encode(value.as_ref())
        .as_str())
}

pub fn deserialize<'de, D>(deserializer: D) -> GitHubResult<Vec<u8>, D::Error>
where D : Deserializer<'de> {

    use base64::{

        engine::general_purpose::{STANDARD},
        Engine,
    };

    String::deserialize(deserializer)
        .and_then(|string| {
            let processed: String = string.chars().filter_map(|character| {
                if character.is_whitespace() { None } else { Some(character) }
            }).collect();

            STANDARD.decode(processed)
                .map_err(|error| Error::custom(error))
        })
}

impl Into<Sha> for Blob {
    fn into(self) -> Sha {
        match self {
            Blob::Binary { sha, .. } => sha.clone(),
            Blob::Text { sha, .. } => sha.clone(),
        }
    }
}