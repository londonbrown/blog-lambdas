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
    pub content_key: String,
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
    pub created_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Content {
    #[serde(rename = "PK")]
    pub pk: String,
    #[serde(rename = "SK")]
    pub sk: String,
    #[serde(rename = "contentType")]
    pub content_type: String,
    pub location: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "authorId")]
    pub author_id: String,
}

fn default_published() -> String {
    "false".to_string()
}

fn default_tags() -> Vec<String> {
    vec![]
}

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    #[serde(rename = "contentKey")]
    pub content_key: String,
    #[serde(default = "default_tags")]
    pub tags: Vec<String>,
    #[serde(default = "default_published")]
    pub published: String,
}

#[derive(Debug, Serialize)]
pub struct CreatePostResponse {
    #[serde(rename = "postId")]
    pub post_id: String,
    pub title: String,
    pub content: String,
    #[serde(rename = "authorId")]
    pub author_id: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateContentRequest {
    #[serde(rename = "contentType")]
    pub content_type: String,
    #[serde(rename = "fileExtension")]
    pub file_extension: String,
    pub body: String,
    #[serde(rename = "isBase64Encoded", default)]
    pub is_base64_encoded: bool,
}

#[derive(Debug, Serialize)]
pub struct CreateContentResponse {
    #[serde(rename = "contentId")]
    pub content_id: String,
    #[serde(rename = "contentType")]
    pub content_type: String,
    pub location: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "authorId")]
    pub author_id: String,
}
