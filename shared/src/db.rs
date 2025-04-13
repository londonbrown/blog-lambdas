use crate::models::{BlogPost, Comment, Content};
use aws_sdk_dynamodb::error::SdkError;
use aws_sdk_dynamodb::operation::get_item::GetItemOutput;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use serde_dynamo::{from_item, from_items, to_item};
use std::collections::HashMap;
use tracing::error;
use tracing::info;

async fn get_item(
    client: &Client,
    table_name: &str,
    partition_key: &str,
) -> Result<GetItemOutput, String> {
    client
        .get_item()
        .table_name(table_name)
        .key("PK", AttributeValue::S(partition_key.to_string()))
        .key("SK", AttributeValue::S("META".to_string()))
        .send()
        .await
        .map_err(|e| format!("DynamoDB error: {}", e))
}

async fn put_item(
    client: &Client,
    table_name: &str,
    item: HashMap<String, AttributeValue>,
) -> Result<(), String> {
    client
        .put_item()
        .table_name(table_name)
        .set_item(Some(item))
        .send()
        .await
        .map_err(|e| match &e {
            SdkError::ServiceError(err) => {
                error!(
                    "DynamoDB service error: {:?}, raw: {:?}",
                    err.err(),
                    err.raw()
                );
                format!("DynamoDB service error: {:?}", err.err())
            }
            SdkError::TimeoutError(err) => {
                error!("DynamoDB timeout error: {:?}", err);
                format!("DynamoDB timeout error: {:?}", err)
            }
            SdkError::ConstructionFailure(err) => {
                error!("DynamoDB request construction failed: {:?}", err);
                format!("DynamoDB request construction failed: {:?}", err)
            }
            SdkError::DispatchFailure(err) => {
                error!("DynamoDB network dispatch failed: {:?}", err);
                format!("DynamoDB network dispatch failed: {:?}", err)
            }
            SdkError::ResponseError(err) => {
                error!("DynamoDB response error: {:?}", err);
                format!("DynamoDB response error: {:?}", err)
            }
            other => {
                error!("Unexpected DynamoDB error: {:?}", other);
                format!("Unexpected DynamoDB error: {:?}", other)
            }
        })?;
    Ok(())
}

pub fn extract_next_token(
    last_evaluated_key: Option<HashMap<String, AttributeValue>>,
) -> Option<String> {
    last_evaluated_key.and_then(|mut key| {
        let pk_value = key.remove("PK")?;
        let sk_value = key.remove("SK")?;

        let pk = pk_value.as_s().ok()?;
        let sk = sk_value.as_s().ok()?;

        Some(format!("{}|{}", pk, sk))
    })
}

pub async fn fetch_post_and_comments(
    client: &Client,
    table_name: &str,
    post_id: &str,
) -> (Option<BlogPost>, Vec<Comment>) {
    let partition_key = format!("POST#{}", post_id);

    let result = client
        .query()
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

pub async fn create_post(client: &Client, table_name: &str, post: &BlogPost) -> Result<(), String> {
    let partition_key = post.pk.clone();

    let existing_post = get_item(client, table_name, &partition_key).await?;

    info!("existing post: {:?}", existing_post);

    if existing_post.item.is_some() {
        return Err("Post already exists".to_string());
    }

    let item = to_item(post).map_err(|e| format!("Serialization error: {}", e))?;

    info!("item: {:?}", item);

    put_item(client, table_name, item).await?;

    Ok(())
}

pub async fn create_content(
    client: &Client,
    table_name: &str,
    content: &Content,
) -> Result<(), String> {
    let partition_key = content.pk.clone();

    let existing_content = get_item(client, table_name, &partition_key).await?;

    info!("existing content: {:?}", existing_content);

    if existing_content.item.is_some() {
        return Err("Content already exists".to_string());
    }

    let item = to_item(content).map_err(|e| format!("Serialization error: {}", e))?;

    info!("item: {:?}", item);

    put_item(client, table_name, item).await?;

    Ok(())
}

pub async fn get_content(
    client: &Client,
    table_name: &str,
    pk: &str,
) -> Result<Option<Content>, String> {
    let item = get_item(client, table_name, pk).await?.item;
    if item.is_none() {
        return Ok(None);
    }
    from_item(item.unwrap()).map_err(|e| format!("Deserialization error: {}", e))
}

pub fn parse_next_token(token: &str) -> Option<HashMap<String, AttributeValue>> {
    let parts: Vec<&str> = token.split('|').collect();
    if parts.len() != 2 {
        return None;
    }

    let mut key = HashMap::new();
    key.insert("PK".to_string(), AttributeValue::S(parts[0].to_string()));
    key.insert("SK".to_string(), AttributeValue::S(parts[1].to_string()));

    Some(key)
}

pub async fn fetch_published_posts(
    client: &Client,
    table_name: &str,
    limit: Option<i32>,
    next_token: Option<String>, // TODO parse next token
) -> (Vec<BlogPost>, Option<String>) {
    let mut request = client
        .query()
        .table_name(table_name)
        .index_name("PublishedIndex")
        .key_condition_expression("published = :published")
        .expression_attribute_values(":published", AttributeValue::S("true".to_string()));

    if let Some(l) = limit {
        request = request.limit(l)
    }

    if let Some(token) = next_token {
        if let Some(start_key) = parse_next_token(&token) {
            request = request.set_exclusive_start_key(Some(start_key))
        }
    }

    let result = request.send().await.expect("DynamoDB query failed");

    let next_token = extract_next_token(result.last_evaluated_key);
    let posts: Vec<BlogPost> = from_items(result.items.unwrap_or_default()).unwrap_or_default();

    (posts, next_token)
}
