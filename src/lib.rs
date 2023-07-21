pub use ::ambient_project;
use ambient_project::{Manifest, Version};

#[cfg(target_arch = "wasm32")]
use firebase_wasm::firestore::{CollectionReference, Timestamp as TimestampRaw};
#[cfg(target_arch = "wasm32")]
use serde_wasm_bindgen::PreserveJsValue;
#[cfg(target_arch = "wasm32")]
type Timestamp = PreserveJsValue<TimestampRaw>;
#[cfg(not(target_arch = "wasm32"))]
use firestore::FirestoreTimestamp as Timestamp;
use parse_display::{Display, FromStr};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DbCollections {
    Embers,
    Profiles,
    ApiKeys,
    Deployments,
    Servers,
    RunningServers,
    Likes,
}
impl DbCollections {
    #[cfg(target_arch = "wasm32")]
    pub fn doc(&self, id: impl AsRef<str>) -> DocRef {
        DocRef(format!("{}/{}", self, id.as_ref()))
    }
    #[cfg(target_arch = "wasm32")]
    pub fn collection(&self) -> CollectionReference {
        let db = firebase_wasm::firestore::get_firestore();
        firebase_wasm::firestore::collection(db, &self.to_string()).unwrap()
    }
}
impl std::fmt::Display for DbCollections {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        serde_plain::to_string(self).unwrap().fmt(f)
    }
}
#[test]
fn test_collections_id() {
    assert_eq!(DbCollections::RunningServers.to_string(), "running_servers");
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DbEmber {
    pub name: String,
    pub owner_id: String,
    pub created: Timestamp,
    pub manifest: Option<Manifest>,
    #[serde(default)]
    pub latest_deployment: String,
    #[serde(default)]
    pub deployments: Vec<String>,
    #[serde(default)]
    pub like_info: LikeInfo,
}

impl DbCollection for DbEmber {
    const COLLECTION: DbCollections = DbCollections::Embers;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DbProfile {
    pub username: String,
    pub created: Timestamp,
}

impl DbCollection for DbProfile {
    const COLLECTION: DbCollections = DbCollections::Profiles;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DbApiKey {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DbDeployment {
    pub ember_id: String,
    pub files: Vec<File>,
    pub manifest: Manifest,
    pub ambient_version: Version,
    pub created: Timestamp,
}

impl DbCollection for DbDeployment {
    const COLLECTION: DbCollections = DbCollections::Deployments;
}

pub type MD5Digest = [u8; 16];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct File {
    pub path: String,
    pub size: usize,
    pub md5: MD5Digest,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DbServer {
    pub name: String,
    pub context: String,
    pub deploy_url: String,
    pub host: String,
    pub state: ServerState,
    pub created: Timestamp,
    pub stopped: Option<Timestamp>,
    #[serde(default)]
    pub logs: Vec<ServerLog>,
}

impl DbCollection for DbServer {
    const COLLECTION: DbCollections = DbCollections::Servers;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum ServerState {
    Starting,
    Running,
    Stopped,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ServerLog {
    pub timestamp: Timestamp,
    pub message: String,
    pub source: Option<ServerLogSource>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum ServerLogSource {
    Stdout,
    Stderr,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DbRunningServer {
    pub server_id: String,
    pub deploy_url: String,
    pub context: String,
}

impl DbRunningServer {
    // NOTE: generated document_id can have a collission with specifically crated values
    pub fn document_id(deploy_url: &str, context: &str) -> String {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update("url:");
        hasher.update(deploy_url);
        hasher.update("context:");
        hasher.update(context);
        let hash = hasher.finalize();
        base16ct::lower::encode_string(&hash)
    }
}

impl DbCollection for DbRunningServer {
    const COLLECTION: DbCollections = DbCollections::RunningServers;
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DbMessage {
    pub user_id: String,
    pub created: Timestamp,
    pub content: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq)]
pub struct LikeInfo {
    pub total: i32,
    pub by_day: Vec<i32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DbLike {
    pub collection: DbCollections,
    pub created: Timestamp,
}

impl DbCollection for DbLike {
    const COLLECTION: DbCollections = DbCollections::Likes;
}
#[derive(Display, FromStr, Debug)]
#[display("{user_id}_{object_id}")]
pub struct DbLikeId {
    pub user_id: String,
    pub object_id: String,
}
