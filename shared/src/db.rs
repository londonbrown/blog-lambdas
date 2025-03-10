use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::types::AttributeValue;
use crate::models::{BlogPost, Comment};
use serde_dynamo::from_item;
use tracing::info;

pub async fn fetch_post_and_comments(client: &Client, table_name: &str, post_id: &str) -> (Option<BlogPost>, Vec<Comment>) {
    let partition_key = format!("POST#{}", post_id);

    let result = client.query()
        .table_name(table_name)
        .key_condition_expression("PK = :pk")
        .expression_attribute_values(":pk", AttributeValue::S(partition_key.to_string()))
        .send()
        .await
        .expect("DynamoDB query failed");

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

    (meta, comments)
}
