use lambda_http::{Body, Request, Response};
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Deserialize, Serialize};
use std::env;
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

pub(crate) async fn function_handler(_event: Request) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new(&aws_config::load_from_env().await);
    let table_name = env::var("BLOG_POSTS_TABLE").expect("BLOG_POSTS_TABLE not set");

    // TODO remove hardcoded post and obtain from event context
    let partition_key = "POST#post-001";

    // Query all items under partition_key
    let result = client.query()
        .table_name(&table_name)
        .key_condition_expression("PK = :pk")
        .expression_attribute_values(":pk", AttributeValue::S(partition_key.to_string()))
        .send()
        .await?;

    info!("DynamoDB response: {:?}", result);

    for item in result.items.unwrap_or_default() {
        if let Some(sk) = item.get("SK").and_then(|v| v.as_s().ok()) {
            if sk == "META" {
                info!("Found META item: {:?}", item)
            } else if sk.starts_with("COMMENT#") {
                info!("Found COMMENT item: {:?}", item)
            }
        }
    }

    Ok(Response::builder()
        .status(200)
        .body("Query executed successfully. Check the logs for item details".into())?)
}
