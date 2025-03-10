use lambda_http::{tracing, Body, Request, Response};
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::types::AttributeValue;
use std::env;
use tracing::info;

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

    Ok(Response::builder()
        .status(200)
        .body("Query executed successfully. Check the logs for item details".into())?)
}
