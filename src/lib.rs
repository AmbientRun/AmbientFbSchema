use ambient_project::Manifest;
use firebase_wasm::firestore::{DocumentReference, Timestamp};
use rs2js::Rs2Js;

#[derive(Debug, Clone, Copy)]
pub enum DbCollections {
    Embers,
    Profiles,
}
impl DbCollections {
    pub fn as_str(&self) -> &'static str {
        match self {
            DbCollections::Embers => "embers",
            DbCollections::Profiles => "profiles",
        }
    }
    pub fn doc(&self, id: impl AsRef<str>) -> DocRef {
        DocRef(format!("{}/{}", self.as_str(), id.as_ref()))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocRef(String);
impl From<DocRef> for DocumentReference {
    fn from(value: DocRef) -> Self {
        let db = firebase_wasm::firestore::get_firestore();
        firebase_wasm::firestore::doc(db, &value.0).unwrap()
    }
}

#[derive(Rs2Js, Debug, Clone, PartialEq)]
pub struct DbEmber {
    pub name: String,
    pub owner_id: String,
    #[raw]
    pub created: Timestamp,
    pub manifest: Manifest,
}

#[derive(Rs2Js, Debug, Clone, PartialEq)]
pub struct DbProfile {
    pub username: String,
    #[raw]
    pub created: Timestamp,
}
