use lambda_http::{Body, Request, RequestExt, Response};
use aws_sdk_dynamodb::Client;
use std::env;
use serde_json::json;
use shared::db::fetch_post_and_comments;

pub(crate) async fn function_handler(event: Request) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new(&aws_config::load_from_env().await);
    let table_name = env::var("BLOG_POSTS_TABLE").expect("BLOG_POSTS_TABLE not set");

    let path_parameters = event.path_parameters();
    let post_id = path_parameters.first("id").unwrap_or_default();

    let (meta, comments) = fetch_post_and_comments(&client, &table_name, post_id).await;

    let response_body = serde_json::to_string(&json!({
        "meta": meta,
        "comments": comments
    }))?;

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(response_body.into())?)
}
