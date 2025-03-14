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
    pub tags: Option<Vec<String>>,
    pub published: Option<String>,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct PostRequest {
    pub title: String,
    pub content: String,
    pub tags: Option<Vec<String>>,
    pub published: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostResponse {
    #[serde(rename = "postId")]
    pub post_id: String,
    pub title: String,
    pub content: String,
    pub author_id: String,
}