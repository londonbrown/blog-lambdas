use std::collections::HashMap;
use std::env;
use lambda_http::{Body, Error, Request, RequestExt, Response};
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::types::AttributeValue;
use serde_json::{json, Value};

fn serialize_dynamodb_item(item: HashMap<String, AttributeValue>) -> HashMap<String, Value> {
    item.into_iter().map(|(key, value)| {
        let json_value = match value {
            AttributeValue::S(s) => Value::String(s),
            AttributeValue::N(n) => Value::String(n),
            AttributeValue::Bool(b) => Value::Bool(b),
            _ => Value::Null
        };
        (key, json_value)
    }).collect()
}

pub(crate) async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let client = Client::new(&aws_config::load_from_env().await);
    let table_name = env::var("BLOG_POSTS_TABLE").expect("BLOG_POSTS_TABLE not set");

    // Extract postId from path parameters

    let post_id = match event.path_parameters().first("id") {
        Some(id) => id.to_string(),
        None => {
            return Ok(Response::builder()
                .status(400)
                .body(json!({"error": "Missing postId"}).to_string().into())?)
        }
    };

    // Retrieve the post from DynamoDB
    let result = client.get_item()
        .table_name(&table_name)
        .key("postId", AttributeValue::S(post_id))
        .send()
        .await?;

    if let Some(item) = result.item {
        let serialized_item = serialize_dynamodb_item(item);
        Ok(Response::builder()
            .status(200)
            .body(serde_json::to_string(&serialized_item)?.into())?)
    } else {
        Ok(Response::builder()
            .status(404)
            .body(json!({"error": "Post not found"}).to_string().into())?)
    }
}