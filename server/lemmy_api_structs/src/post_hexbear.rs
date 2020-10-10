use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GetFeaturedPosts {
    pub auth: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FeaturePost {
    pub id: i32,
    pub featured: bool,
    pub auth: String,
}
