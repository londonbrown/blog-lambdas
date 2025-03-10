use lambda_http::{Body, Request, RequestExt, Response};
use aws_sdk_dynamodb::Client;
use std::env;
use serde_json::json;
use shared::api::with_cors;
use shared::db::{fetch_published_posts};
use shared::errors::ApiErrorResponse;

pub(crate) async fn function_handler(event: Request) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
    if event.method() == lambda_http::http::Method::OPTIONS {
        return Ok(with_cors(Response::builder()
            .status(204)
            .body(Body::Empty)?)
            .unwrap_or_else(|err_response| err_response))
    }

    let client = Client::new(&aws_config::load_from_env().await);
    let table_name = env::var("BLOG_POSTS_TABLE").expect("BLOG_POSTS_TABLE not set");

    let query_string_parameters = event.query_string_parameters();
    let limit = query_string_parameters.first("limit").and_then(|l| l.parse().ok());
    let next_token: Option<String> = None; // TODO Extract from request

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

    Ok(with_cors(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&response_body)?.into())?)
        .unwrap_or_else(|err_response| err_response))
}
