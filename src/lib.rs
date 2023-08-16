#[cfg(target_arch = "wasm32")]
use firebase_wasm::firestore::{CollectionReference, Timestamp as TimestampRaw};
#[cfg(target_arch = "wasm32")]
use serde_wasm_bindgen::PreserveJsValue;
#[cfg(target_arch = "wasm32")]
type Timestamp = PreserveJsValue<TimestampRaw>;
pub use ambient_project::EmberContent;
#[cfg(not(target_arch = "wasm32"))]
use firestore::FirestoreTimestamp as Timestamp;
use parse_display::{Display, FromStr};
use serde::{Deserialize, Serialize};
use serde_plain::{derive_display_from_serialize, derive_fromstr_from_deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DbCollections {
    Embers,
    Profiles,
    ApiKeys,
    Deployments,
    Servers,
    ServerLogs,
    RunningServers,
    Upvotes,
    Activities,
}
impl DbCollections {
    #[cfg(target_arch = "wasm32")]
    pub fn doc(
        &self,
        id: impl AsRef<str>,
    ) -> Result<firebase_wasm::firestore::DocumentReference, String> {
        let db = firebase_wasm::firestore::get_firestore();
        firebase_wasm::firestore::doc(db, &format!("{}/{}", self, id.as_ref()))
            .map_err(|x| x.as_string().unwrap_or_else(|| "unknown error".into()))
    }
    #[cfg(target_arch = "wasm32")]
    pub fn collection(&self) -> CollectionReference {
        let db = firebase_wasm::firestore::get_firestore();
        firebase_wasm::firestore::collection(db, &self.to_string()).unwrap()
    }
}
derive_display_from_serialize!(DbCollections);
derive_fromstr_from_deserialize!(DbCollections);

#[test]
fn test_collections_id() {
    assert_eq!(DbCollections::RunningServers.to_string(), "running_servers");
}

pub trait DbCollection {
    const COLLECTION: DbCollections;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DbEmber {
    pub owner_id: String,
    pub created: Timestamp,
    pub updated: Timestamp,
    #[serde(default)]
    pub latest_deployment: String,
    #[serde(default)]
    pub deployments: Vec<String>,
    /// If this is featured by ambient
    pub featured: Option<f32>,
    #[serde(default)]
    pub total_upvotes: i32,
    #[serde(default)]
    pub latest_screenshot_url: String,
    #[serde(default)]
    pub latest_readme_url: String,
    /// If true; this can be deleted 24h after it was created
    #[serde(default)]
    pub temporary: bool,

    // Information pulled from the `ambient.toml`:
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub content: DbEmberContent,
}

impl DbCollection for DbEmber {
    const COLLECTION: DbCollections = DbCollections::Embers;
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DbEmberContent {
    pub playable: bool,
    pub example: bool,
    pub asset: bool,
    pub models: bool,
    pub textures: bool,
    pub fonts: bool,
    pub code: bool,
    pub audio: bool,
    pub tool: bool,
    pub mod_: bool,
}
impl DbEmberContent {
    pub fn from_content(content: &EmberContent) -> Self {
        Self {
            playable: matches!(content, EmberContent::Playable { example: _ }),
            example: matches!(content, EmberContent::Playable { example: true }),
            asset: matches!(
                content,
                EmberContent::Asset {
                    models: _,
                    textures: _,
                    audio: _,
                    fonts: _,
                    code: _
                }
            ),
            models: matches!(
                content,
                EmberContent::Asset {
                    models: true,
                    textures: _,
                    audio: _,
                    fonts: _,
                    code: _
                }
            ),
            textures: matches!(
                content,
                EmberContent::Asset {
                    models: _,
                    textures: true,
                    audio: _,
                    fonts: _,
                    code: _
                }
            ),
            audio: matches!(
                content,
                EmberContent::Asset {
                    models: _,
                    textures: _,
                    audio: true,
                    fonts: _,
                    code: _
                }
            ),
            fonts: matches!(
                content,
                EmberContent::Asset {
                    models: _,
                    textures: _,
                    audio: _,
                    fonts: true,
                    code: _
                }
            ),
            code: matches!(
                content,
                EmberContent::Asset {
                    models: _,
                    textures: _,
                    audio: _,
                    fonts: _,
                    code: true
                }
            ),
            tool: matches!(content, EmberContent::Tool),
            mod_: matches!(content, EmberContent::Mod { for_playables: _ }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DbProfile {
    pub username: String,
    pub display_name: String,
    pub bio: String,
    pub github: String,
    pub twitter: String,
    pub instagram: String,
    pub linkedin: String,
    pub twitch: String,
    pub website: String,
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
    /// The user that deployed this
    #[serde(default)]
    pub user_id: String,
    pub files: Vec<File>,
    pub ambient_version: String,
    #[serde(default)]
    pub ambient_revision: String,
    pub created: Timestamp,
    #[serde(default)]
    pub has_screenshot: bool,
    #[serde(default)]
    pub has_readme: bool,
    /// If true; this can be deleted 24h after it was created
    #[serde(default)]
    pub temporary: bool,

    // Information pulled from the `ambient.toml`:
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub ember_type: EmberType,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub enum EmberType {
    #[default]
    Game,
    Mod,
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

/// A log entry from a server
/// Subcollection of DbServer
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct DbServerLog {
    pub timestamp: Timestamp,
    pub message: String,
    pub source: Option<ServerLogSource>,
}

impl DbCollection for DbServerLog {
    const COLLECTION: DbCollections = DbCollections::ServerLogs;
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
pub struct DbUpvotable {
    pub total_upvotes: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DbUpvote {
    pub collection: DbCollections,
    pub created: Timestamp,
    pub user_id: String,
    pub item_id: String,
}

impl DbCollection for DbUpvote {
    const COLLECTION: DbCollections = DbCollections::Upvotes;
}
#[derive(Display, FromStr, Debug)]
#[display("{user_id}_{object_id}")]
pub struct DbUpvoteId {
    pub user_id: String,
    pub object_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Activity {
    EmberDeployed {
        ember_id: String,
        deployment_id: String,
    },
    MessagePosted {
        path: String,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DbActivity {
    pub timestamp: Timestamp,
    pub content: Activity,
}
impl DbCollection for DbActivity {
    const COLLECTION: DbCollections = DbCollections::Activities;
}
