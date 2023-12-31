#[cfg(target_arch = "wasm32")]
use firebase_wasm::firestore::{CollectionReference, Timestamp as TimestampRaw};
#[cfg(target_arch = "wasm32")]
use serde_wasm_bindgen::PreserveJsValue;
#[cfg(target_arch = "wasm32")]
type Timestamp = PreserveJsValue<TimestampRaw>;
pub use ambient_package::PackageContent;
#[cfg(not(target_arch = "wasm32"))]
use firestore::FirestoreTimestamp as Timestamp;
use parse_display::{Display, FromStr};
use serde::{Deserialize, Serialize};
use serde_plain::{derive_display_from_serialize, derive_fromstr_from_deserialize};
use sha2::digest::OutputSizeUser;
use ts_rs::TS;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum DbCollections {
    Packages,
    PackageVersions,
    Profiles,
    ApiKeys,
    Deployments,
    Servers,
    ServerLogs,
    RunningServers,
    ShardedServers,
    Upvotes,
    Activities,
}
impl DbCollections {
    #[cfg(target_arch = "wasm32")]
    pub fn doc(
        &self,
        id: impl AsRef<str>,
    ) -> anyhow::Result<firebase_wasm::firestore::DocumentReference> {
        let db = firebase_wasm::firestore::get_firestore();
        let doc = firebase_wasm::firestore::doc(db, &format!("{}/{}", self, id.as_ref()));
        match doc {
            Ok(doc) => Ok(doc),
            Err(err) => Err(anyhow::anyhow!(
                "Failed to create doc ref: {}",
                err.as_string()
                    .unwrap_or_else(|| "unknown error".to_string())
            )),
        }
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DbPackage {
    pub owner_id: String,
    #[ts(type = "Timestamp")]
    pub created: Timestamp,
    #[ts(type = "Timestamp")]
    pub updated: Timestamp,
    #[serde(default)]
    pub deleted: bool,
    #[serde(default)]
    pub latest_version: Option<DbPackageVersionWithVersion>,
    #[serde(default)]
    pub latest_deployment: String,
    #[serde(default)]
    pub deployments: Vec<String>,
    /// If this is featured by ambient
    pub featured: Option<f32>,
    #[serde(default)]
    pub latest_screenshot_url: String,
    #[serde(default)]
    pub latest_readme_url: String,
    #[serde(default)]
    pub total_upvotes: i32,

    // Information pulled from the `ambient.toml`:
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, deserialize_with = "deserialize_package_content")]
    pub content: Vec<DbPackageContent>,
    #[serde(default)]
    pub for_playables: Vec<String>,
    #[serde(default)]
    pub public: bool,
}

impl DbCollection for DbPackage {
    const COLLECTION: DbCollections = DbCollections::Packages;
}

impl DbPackage {
    /// Helper method for getting the parent path for a given package_id,
    /// so that it can be used with DbPackageVersion / `parent`
    #[cfg(not(target_arch = "wasm32"))]
    pub fn parent_path_for(
        db: &firestore::FirestoreDb,
        package_id: &str,
    ) -> firestore::FirestoreResult<String> {
        Ok(db
            .parent_path(&Self::COLLECTION.to_string(), package_id)?
            .to_string())
    }
}

/// Subcollection of DbPackage
#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct DbPackageVersion {
    pub deployment_id: String,
}

impl DbCollection for DbPackageVersion {
    const COLLECTION: DbCollections = DbCollections::PackageVersions;
}

/// Helper struct used with `DbPackage` to store the latest version
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DbPackageVersionWithVersion {
    pub version: String,
    pub deployment_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DbDeletable {
    #[serde(default)]
    pub deleted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Display, FromStr, TS)]
#[ts(export)]
pub enum DbPackageContent {
    Playable,
    Example,
    NotExample,
    Asset,
    Models,
    Animations,
    Textures,
    Materials,
    Fonts,
    Code,
    Schema,
    Audio,
    Other,
    Tool,
    Mod,
}
impl DbPackageContent {
    pub fn from_content(content: &PackageContent) -> Vec<Self> {
        match content {
            PackageContent::Playable { example: false } => vec![Self::Playable, Self::NotExample],
            PackageContent::Playable { example: true } => vec![Self::Playable, Self::Example],
            PackageContent::Asset {
                models,
                animations,
                textures,
                materials,
                audio,
                fonts,
                code,
                schema,
            } => {
                let mut res = vec![Self::Asset];
                if *models {
                    res.push(Self::Models);
                }
                if *animations {
                    res.push(Self::Animations);
                }
                if *textures {
                    res.push(Self::Textures);
                }
                if *materials {
                    res.push(Self::Materials);
                }
                if *audio {
                    res.push(Self::Audio);
                }
                if *fonts {
                    res.push(Self::Fonts);
                }
                if *code {
                    res.push(Self::Code);
                }
                if *schema {
                    res.push(Self::Schema);
                }
                if res.len() == 1 {
                    res.push(Self::Other);
                }
                res
            }
            PackageContent::Tool => vec![Self::Tool],
            PackageContent::Mod { for_playables: _ } => vec![Self::Mod],
        }
    }

    pub fn from_legacy_db_package_content(value: LegacyDbPackageContent) -> Vec<Self> {
        let mut content = Vec::new();
        let LegacyDbPackageContent {
            playable,
            example,
            asset,
            models,
            animations,
            textures,
            materials,
            fonts,
            code,
            schema,
            audio,
            tool,
            mod_,
        } = value;
        if playable {
            content.push(Self::Playable);
            if example {
                content.push(Self::Example);
            } else {
                content.push(Self::NotExample);
            }
        }
        if asset {
            content.push(Self::Asset);
            if models {
                content.push(Self::Models);
            }
            if animations {
                content.push(Self::Animations);
            }
            if textures {
                content.push(Self::Textures);
            }
            if materials {
                content.push(Self::Materials);
            }
            if fonts {
                content.push(Self::Fonts);
            }
            if code {
                content.push(Self::Code);
            }
            if schema {
                content.push(Self::Schema);
            }
            if audio {
                content.push(Self::Audio);
            }
        }
        if tool {
            content.push(Self::Tool);
        }
        if mod_ {
            content.push(Self::Mod);
        }
        if content.is_empty() {
            content.push(Self::Other);
        }
        content
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct LegacyDbPackageContent {
    #[serde(default)]
    pub playable: bool,
    #[serde(default)]
    pub example: bool,
    #[serde(default)]
    pub asset: bool,
    #[serde(default)]
    pub models: bool,
    #[serde(default)]
    pub animations: bool,
    #[serde(default)]
    pub textures: bool,
    #[serde(default)]
    pub materials: bool,
    #[serde(default)]
    pub fonts: bool,
    #[serde(default)]
    pub code: bool,
    #[serde(default)]
    pub schema: bool,
    #[serde(default)]
    pub audio: bool,
    #[serde(default)]
    pub tool: bool,
    #[serde(default)]
    pub mod_: bool,
}
fn deserialize_package_content<'de, D>(deserializer: D) -> Result<Vec<DbPackageContent>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Clone, Debug, Deserialize)]
    #[serde(untagged)]
    pub enum DbPackageContentShim {
        New(Vec<DbPackageContent>),
        Legacy(LegacyDbPackageContent),
    }

    Ok(match DbPackageContentShim::deserialize(deserializer)? {
        DbPackageContentShim::New(v) => v,
        DbPackageContentShim::Legacy(v) => DbPackageContent::from_legacy_db_package_content(v),
    })
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DbProfile {
    pub created: Timestamp,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub bio: String,
    #[serde(default)]
    pub github: String,
    #[serde(default)]
    pub twitter: String,
    #[serde(default)]
    pub instagram: String,
    #[serde(default)]
    pub linkedin: String,
    #[serde(default)]
    pub twitch: String,
    #[serde(default)]
    pub website: String,
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
    pub package_id: String,
    /// The user that deployed this
    #[serde(default)]
    pub user_id: String,
    pub files: Vec<File>,
    pub ambient_version: semver::Version,
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
    #[serde(default = "zero_version")]
    pub version: semver::Version,
    #[serde(default)]
    pub content: PackageContent,
}
fn zero_version() -> semver::Version {
    semver::Version::new(0, 0, 0)
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Region {
    EU,
    US,
}

derive_display_from_serialize!(Region);
derive_fromstr_from_deserialize!(Region);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DbServer {
    pub context: String,
    pub deploy_url: String,
    pub host: String,
    pub state: ServerState,
    pub created: Timestamp,
    pub updated: Timestamp,
    pub player_count: Option<u32>,
    pub region: Region,
    #[serde(default)]
    pub package_id: String,
    #[serde(default)]
    pub deployment_id: String,
    #[serde(default)]
    pub owner_id: String,
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
    #[serde(default)]
    pub server_id: String,
    pub deploy_url: String,
    pub context: String,
    pub region: Region,
    #[serde(default)]
    pub package_id: String,
    #[serde(default)]
    pub deployment_id: String,
    #[serde(default)]
    pub owner_id: String,
}

impl DbRunningServer {
    // NOTE: generated document_id can have a collission with specifically created values
    pub fn document_id(
        region: Region,
        fleet: Option<&str>,
        deploy_url: &str,
        context: &str,
    ) -> String {
        use sha2::{Digest, Sha256};

        if let Some(fleet) = fleet {
            assert!(has_only_safe_chars(fleet));
        }

        let hash_char_len = <Sha256 as OutputSizeUser>::output_size() * 2; // 2 because each byte becomes 2 chars in hex

        // fleet_str is "-FLEET" or ""
        let fleet_str = fleet
            .and_then(|f| (!f.is_empty()).then(|| format!("-{}", f)))
            .unwrap_or_default();
        let region_with_fleet = format!("{}{}", region, fleet_str);
        match (
            deploy_url.strip_prefix("https://assets.ambient.run/"),
            context,
        ) {
            (Some(id), "") if has_only_safe_chars(id) => format!("{}-{}", region_with_fleet, id),
            (Some(id), ctx)
                if has_only_safe_chars(id)
                    && ctx.len() < hash_char_len
                    && has_only_safe_chars(ctx) =>
            {
                format!("{}-{}-{}", region_with_fleet, id, ctx)
            }
            _ => {
                let mut hasher = Sha256::new();
                hasher.update("url:");
                hasher.update(deploy_url);
                hasher.update("context:");
                hasher.update(context);
                let hash = hasher.finalize();
                let hash = base16ct::lower::encode_string(&hash);
                format!("{}-{}", region_with_fleet, hash)
            }
        }
    }
}

fn has_only_safe_chars(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_alphanumeric())
}

#[test]
fn test_running_server_document_id() {
    assert_eq!(
        &DbRunningServer::document_id(
            Region::EU,
            None,
            "https://assets.ambient.run/somedeployment",
            ""
        ),
        "EU-somedeployment"
    );
    assert_eq!(
        &DbRunningServer::document_id(
            Region::EU,
            None,
            "https://assets.ambient.run/somedeployment",
            "context"
        ),
        "EU-somedeployment-context"
    );
    assert_eq!(
        &DbRunningServer::document_id(
            Region::EU,
            None,
            "https://assets.ambient.run/somedeployment",
            std::str::from_utf8(&[b'A'; 128]).unwrap()
        ),
        "EU-0d0b59652cc8bb5ed48964f804f3f00c731e9f90f4f814bd3add922c214cfa62"
    );
    assert_eq!(
        &DbRunningServer::document_id(
            Region::EU,
            Some("canary"),
            "https://assets.ambient.run/somedeployment",
            ""
        ),
        "EU-canary-somedeployment"
    );
    assert_eq!(
        &DbRunningServer::document_id(
            Region::EU,
            Some("canary"),
            "https://assets.ambient.run/somedeployment",
            "context"
        ),
        "EU-canary-somedeployment-context"
    );
}

impl DbCollection for DbRunningServer {
    const COLLECTION: DbCollections = DbCollections::RunningServers;
}

/// Subcollection of DbRunningServer
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DbShardedServer {
    pub server_id: String,
}

impl DbCollection for DbShardedServer {
    const COLLECTION: DbCollections = DbCollections::ShardedServers;
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct DbMessage {
    pub user_id: String,
    #[ts(type = "Timestamp")]
    pub created: Timestamp,
    pub content: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct DbUpvote {
    pub collection: DbCollections,
    #[ts(type = "Timestamp")]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, TS)]
#[ts(export)]
pub struct DbUpvotable {
    #[serde(default)]
    pub total_upvotes: i32,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, TS)]
#[serde(tag = "type")]
#[ts(export)]
pub enum Activity {
    PackageDeployed {
        package_id: String,
        deployment_id: String,
        version: Option<String>,
    },
    MessagePosted {
        path: String,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct DbActivity {
    #[ts(type = "Timestamp")]
    pub timestamp: Timestamp,
    #[serde(default)]
    pub user_id: String,
    pub content: Activity,
}
impl DbCollection for DbActivity {
    const COLLECTION: DbCollections = DbCollections::Activities;
}
