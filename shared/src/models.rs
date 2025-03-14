use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct BlogPost {
    #[serde(rename = "PK")]
    pub pk: String,
    #[serde(rename = "SK")]
    pub sk: String,
    pub title: String,
    #[serde(rename = "authorId")]
    pub author_id: String,
    pub tags: Vec<String>,
    pub published: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "contentKey")]
    pub content_key: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Comment {
    #[serde(rename = "PK")]
    pub pk: String,
    #[serde(rename = "SK")]
    pub sk: String,
    #[serde(rename = "userId")]
    pub user_id: String,
    pub text: String,
    #[serde(rename = "createdAt")]
    pub created_at: String
}

fn default_published() -> String {
    "false".to_string()
}

fn default_tags() -> Vec<String> {
    vec![]
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PostRequest {
    pub title: String,
    pub content: String,
    #[serde(default = "default_tags")]
    pub tags: Vec<String>,
    #[serde(default = "default_published")]
    pub published: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostResponse {
    #[serde(rename = "postId")]
    pub post_id: String,
    pub title: String,
    pub content: String,
    pub author_id: String,
}