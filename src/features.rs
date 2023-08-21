/// Features are building blocks to firebase documents
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FeatNamed {
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FeatUpvotable {
    #[serde(default)]
    pub total_upvotes: i32,
}
