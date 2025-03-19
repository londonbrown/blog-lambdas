use aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use aws_lambda_events::encodings::Body;
use aws_lambda_events::http::header::CONTENT_TYPE;
use aws_lambda_events::http::HeaderMap;
use aws_sdk_dynamodb::Client;
use lambda_runtime::LambdaEvent;
use serde_json::json;
use shared::db::fetch_published_posts;
use shared::errors::ApiErrorResponse;
use std::env;

pub(crate) async fn function_handler(
    event: LambdaEvent<ApiGatewayProxyRequest>,
) -> Result<ApiGatewayProxyResponse, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new(&aws_config::load_from_env().await);
    let table_name = env::var("BLOG_POSTS_TABLE").expect("BLOG_POSTS_TABLE not set");

    let query_string_parameters = event.payload.query_string_parameters;

    let mut header_map = HeaderMap::new();
    header_map.insert(CONTENT_TYPE, "application/json".parse().unwrap());

    let limit = query_string_parameters
        .first("limit")
        .and_then(|l| l.parse().ok());
    let next_token = query_string_parameters
        .first("nextToken")
        .map(|s| s.to_string());

    let (posts, next_token) = fetch_published_posts(&client, &table_name, limit, next_token).await;

    if posts.is_empty() {
        let error = serde_json::to_string(&ApiErrorResponse::new("No posts found"))?;
        return Ok(ApiGatewayProxyResponse {
            status_code: 404,
            headers: header_map,
            body: Some(Body::Text(error)),
            ..Default::default()
        });
    }

    let response_body = json!({
        "posts": posts,
        "nextToken": next_token
    });

    Ok(ApiGatewayProxyResponse {
        status_code: 200,
        headers: header_map,
        body: Some(Body::Text(response_body.to_string())),
        ..Default::default()
    })
}
