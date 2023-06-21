pub use ::ambient_project;
use ambient_project::{Manifest, Version};
#[cfg(target_arch = "wasm32")]
use rs2js::Rs2Js;

#[cfg(target_arch = "wasm32")]
use firebase_wasm::firestore::Timestamp;
#[cfg(not(target_arch = "wasm32"))]
use firestore::FirestoreTimestamp as Timestamp;

#[derive(Debug, Clone, Copy)]
pub enum DbCollections {
    Embers,
    Profiles,
    ApiKeys,
    Deployments,
}
impl DbCollections {
    pub fn as_str(&self) -> &'static str {
        match self {
            DbCollections::Embers => "embers",
            DbCollections::Profiles => "profiles",
            DbCollections::ApiKeys => "api_keys",
            DbCollections::Deployments => "deployments",
        }
    }
    #[cfg(target_arch = "wasm32")]
    pub fn doc(&self, id: impl AsRef<str>) -> DocRef {
        DocRef(format!("{}/{}", self.as_str(), id.as_ref()))
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone, PartialEq)]
pub struct DocRef(String);

#[cfg(target_arch = "wasm32")]
impl From<DocRef> for firebase_wasm::firestore::DocumentReference {
    fn from(value: DocRef) -> Self {
        let db = firebase_wasm::firestore::get_firestore();
        firebase_wasm::firestore::doc(db, &value.0).unwrap()
    }
}

pub trait DbCollection {
    const COLLECTION: DbCollections;
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(Rs2Js))]
#[cfg_attr(
    not(target_arch = "wasm32"),
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct DbEmber {
    pub name: String,
    pub owner_id: String,
    #[cfg_attr(target_arch = "wasm32", raw)]
    pub created: Timestamp,
    pub manifest: Option<Manifest>,
    #[cfg_attr(not(target_arch = "wasm32"), serde(default))]
    pub latest_deployment: String,
    #[cfg_attr(not(target_arch = "wasm32"), serde(default))]
    pub deployments: Vec<String>,
}

impl DbCollection for DbEmber {
    const COLLECTION: DbCollections = DbCollections::Embers;
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(Rs2Js))]
#[cfg_attr(
    not(target_arch = "wasm32"),
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct DbProfile {
    pub username: String,
    #[cfg_attr(target_arch = "wasm32", raw)]
    pub created: Timestamp,
}

impl DbCollection for DbProfile {
    const COLLECTION: DbCollections = DbCollections::Profiles;
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(Rs2Js))]
#[cfg_attr(
    not(target_arch = "wasm32"),
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct DbApiKey {
    #[cfg_attr(target_arch = "wasm32", raw)]
    pub created: Timestamp,
    pub user_id: String,
    pub name: String,
}

impl DbCollection for DbApiKey {
    const COLLECTION: DbCollections = DbCollections::ApiKeys;
}

pub fn hash_api_key(api_key: &str) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update("ambient");
    hasher.update(api_key);
    let hash = hasher.finalize();
    base16ct::lower::encode_string(&hash)
}

#[derive(Debug, Clone)]
#[cfg_attr(target_arch = "wasm32", derive(Rs2Js))]
#[cfg_attr(
    not(target_arch = "wasm32"),
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct DbDeployment {
    pub ember_id: String,
    pub files: Vec<File>,
    pub manifest: Manifest,
    pub ambient_version: Version,
    #[cfg_attr(target_arch = "wasm32", raw)]
    pub created: Timestamp,
}

impl DbCollection for DbDeployment {
    const COLLECTION: DbCollections = DbCollections::Deployments;
}

pub type MD5Digest = [u8; 16];

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct File {
    pub path: String,
    pub size: usize,
    pub md5: MD5Digest,
}
