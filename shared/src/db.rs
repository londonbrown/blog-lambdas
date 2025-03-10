use std::collections::HashMap;
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::types::AttributeValue;
use crate::models::{BlogPost, Comment};
use serde_dynamo::{from_item, from_items};
use tracing::info;

pub fn extract_next_token(last_evaluated_key: Option<HashMap<String, AttributeValue>>) -> Option<String> {
    last_evaluated_key.and_then(|mut key| {
        let pk_value = key.remove("PK")?;
        let sk_value = key.remove("SK")?;

        let pk = pk_value.as_s().ok()?;
        let sk = sk_value.as_s().ok()?;

        Some(format!("{}|{}", pk, sk))
    })
}


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

pub async fn fetch_published_posts(
    client: &Client,
    table_name: &str,
    limit: Option<i32>,
    _next_token: Option<String> // TODO parse next token
) -> (Vec<BlogPost>, Option<String>) {
    let mut request = client.query()
        .table_name(table_name)
        .index_name("PublishedIndex")
        .key_condition_expression("published = :published")
        .expression_attribute_values(":published", AttributeValue::S("true".to_string()));

    if let Some(l) = limit {
        request = request.limit(l)
    }

    let result = request.send().await.expect("DynamoDB query failed");

    let next_token = extract_next_token(result.last_evaluated_key);
    let posts: Vec<BlogPost> = from_items(result.items.unwrap_or_default()).unwrap_or_default();

    (posts, next_token)
}
