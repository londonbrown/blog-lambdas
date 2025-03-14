use lambda_http::{Body, Request, RequestExt, Response};
use aws_sdk_dynamodb::Client;
use std::env;
use serde_json::json;
use shared::db::{fetch_published_posts};
use shared::errors::ApiErrorResponse;

pub(crate) async fn function_handler(event: Request) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new(&aws_config::load_from_env().await);
    let table_name = env::var("BLOG_POSTS_TABLE").expect("BLOG_POSTS_TABLE not set");

    let query_string_parameters = event.query_string_parameters();
    let limit = query_string_parameters.first("limit").and_then(|l| l.parse().ok());
    let next_token = query_string_parameters.first("nextToken").map(|s| s.to_string());

    let (posts, next_token) = fetch_published_posts(&client, &table_name, limit, next_token).await;

    if posts.is_empty() {
        return Ok(Response::builder()
            .status(404)
            .body(serde_json::to_string(&ApiErrorResponse::new("No posts found"))?.into())?);
    }

    let response_body = json!({
        "posts": posts,
        "nextToken": next_token
    });

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&response_body)?.into())?)
}
