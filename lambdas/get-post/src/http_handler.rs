use lambda_http::{Body, Request, RequestExt, Response};
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Deserialize, Serialize};
use std::env;
use serde_dynamo::from_item;
use serde_json::json;
use tracing::info;

// TODO make this struct a shared object
#[derive(Debug, Deserialize, Serialize)]
struct BlogPost {
    #[serde(rename = "PK")]
    pk: String,
    #[serde(rename = "SK")]
    sk: String,
    title: String,
    #[serde(rename = "authorId")]
    author_id: String,
    tags: Option<Vec<String>>,
    published: Option<String>,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(rename = "contentKey")]
    content_key: String
}

// TODO make this struct a shared object
#[derive(Debug, Deserialize, Serialize)]
struct Comment {
    #[serde(rename = "PK")]
    pk: String,
    #[serde(rename = "SK")]
    sk: String,
    #[serde(rename = "userId")]
    user_id: String,
    text: String,
    #[serde(rename = "createdAt")]
    created_at: String
}

pub(crate) async fn function_handler(event: Request) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new(&aws_config::load_from_env().await);
    let table_name = env::var("BLOG_POSTS_TABLE").expect("BLOG_POSTS_TABLE not set");

    let path_parameters = event.path_parameters();
    let post_id = path_parameters.first("id").unwrap_or_default();
    let partition_key = format!("POST#{}", post_id);

    // Query all items under partition_key
    let result = client.query()
        .table_name(&table_name)
        .key_condition_expression("PK = :pk")
        .expression_attribute_values(":pk", AttributeValue::S(partition_key.to_string()))
        .send()
        .await?;

    let mut meta: Option<BlogPost> = None;
    let mut comments: Vec<Comment> = Vec::new();

    for raw_item in result.items.unwrap_or_default() {
        if let Some(sk) = raw_item.get("SK").and_then(|v| v.as_s().ok()) {
            if sk == "META" {
                match from_item(raw_item) {
                    Ok(post) => meta = Some(post),
                    Err(e) => info!("Failed to deserialize BlogPost: {:?}", e),
                }
            } else if sk.starts_with("COMMENT#") {
                match from_item(raw_item) {
                    Ok(comment) => comments.push(comment),
                    Err(e) => info!("Failed to deserialize Comment: {:?}", e),
                }
            }
        }
    }

    let response_body = serde_json::to_string(&json!({
        "meta": meta,
        "comments": comments
    }))?;

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(response_body.into())?)
}
