use aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use aws_lambda_events::encodings::Body;
use aws_lambda_events::http::header::CONTENT_TYPE;
use aws_lambda_events::http::HeaderMap;
use aws_sdk_dynamodb::Client;
use lambda_runtime::LambdaEvent;
use serde_json::json;
use shared::db::fetch_post_and_comments;
use shared::errors::ApiErrorResponse;
use std::env;

pub(crate) async fn function_handler(
    event: LambdaEvent<ApiGatewayProxyRequest>,
) -> Result<ApiGatewayProxyResponse, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new(&aws_config::load_from_env().await);
    let table_name = env::var("BLOG_POSTS_TABLE").expect("BLOG_POSTS_TABLE not set");

    let path_parameters = event.payload.path_parameters;
    let post_id = path_parameters.get("id").ok_or("Missing post id")?;
    if post_id.is_empty() {
        return Ok(ApiGatewayProxyResponse {
            status_code: 400,
            body: Some(Body::Text(serde_json::to_string(&ApiErrorResponse::new(
                "Missing post id",
            ))?)),
            ..Default::default()
        });
    }

    let (meta, comments) = fetch_post_and_comments(&client, &table_name, post_id).await;

    if meta.is_none() {
        return Ok(ApiGatewayProxyResponse {
            status_code: 404,
            body: Some(Body::Text(serde_json::to_string(&ApiErrorResponse::new(
                "Post not found",
            ))?)),
            ..Default::default()
        });
    }

    let response_body = json!({
        "meta": meta,
        "comments": comments
    });

    let mut header_map = HeaderMap::new();
    header_map.insert(CONTENT_TYPE, "application/json".parse().unwrap());

    Ok(ApiGatewayProxyResponse {
        status_code: 200,
        headers: header_map,
        body: Some(Body::Text(response_body.to_string())),
        ..Default::default()
    })
}
