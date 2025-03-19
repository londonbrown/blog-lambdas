use aws_sdk_dynamodb::Client;
use lambda_http::{Body, Request, RequestExt, Response};
use serde_json::json;
use shared::db::fetch_post_and_comments;
use shared::errors::ApiErrorResponse;
use std::env;

pub(crate) async fn function_handler(
    event: Request,
) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new(&aws_config::load_from_env().await);
    let table_name = env::var("BLOG_POSTS_TABLE").expect("BLOG_POSTS_TABLE not set");

    let path_parameters = event.path_parameters();
    let post_id = path_parameters.first("id").unwrap_or_default();
    if post_id.is_empty() {
        return Ok(Response::builder()
            .status(400)
            .body(serde_json::to_string(&ApiErrorResponse::new("Missing post id"))?.into())?);
    }

    let (meta, comments) = fetch_post_and_comments(&client, &table_name, post_id).await;

    if meta.is_none() {
        return Ok(Response::builder()
            .status(404)
            .body(serde_json::to_string(&ApiErrorResponse::new("Post not found"))?.into())?);
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
