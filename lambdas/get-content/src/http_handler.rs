use aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use aws_lambda_events::encodings::Body;
use aws_lambda_events::http::HeaderMap;
use aws_sdk_dynamodb::Client as DdbClient;
use aws_sdk_s3::Client as S3Client;
use base64::engine::general_purpose::STANDARD as base64_engine;
use base64::Engine as _;
use lambda_runtime::LambdaEvent;
use shared::db::get_content;
use shared::errors::ApiErrorResponse;
use shared::models::Content;
use std::env;

pub(crate) async fn function_handler(
    event: LambdaEvent<ApiGatewayProxyRequest>,
) -> Result<ApiGatewayProxyResponse, Box<dyn std::error::Error + Send + Sync>> {
    let table_name = env::var("BLOG_CONTENT_TABLE").expect("BLOG_CONTENT_TABLE not set");
    let bucket = env::var("BLOG_CONTENT_BUCKET").expect("BLOG_CONTENT_BUCKET not set");

    let request = event.payload;
    let content_id = request
        .path_parameters
        .get("id")
        .ok_or("Missing content id path parameter")?;

    let pk = format!("CONTENT#{}", content_id);

    let ddb = DdbClient::new(&aws_config::load_from_env().await);
    let maybe_content: Option<Content> = get_content(&ddb, &table_name, &pk).await?;

    if maybe_content.is_none() {
        let error = serde_json::to_string(&ApiErrorResponse::new("Content not found"))?;
        return Ok(ApiGatewayProxyResponse {
            status_code: 404,
            body: Some(Body::Text(error)),
            ..Default::default()
        });
    }

    let content = maybe_content.unwrap();

    let s3 = S3Client::new(&aws_config::load_from_env().await);
    let key = content
        .location
        .strip_prefix(&format!("s3://{}/", bucket))
        .ok_or("Invalid content location path")?
        .to_string();

    let result = s3.get_object().bucket(&bucket).key(&key).send().await?;
    let bytes = result.body.collect().await?.into_bytes();
    let content_type = content.content_type;

    let (body, is_base64_encoded) =
        if content_type.starts_with("text/") || content_type == "application/json" {
            (Body::Text(String::from_utf8(bytes.to_vec())?), false)
        } else {
            let base64 = base64_engine.encode(&bytes);
            (Body::Text(base64), true)
        };

    Ok(ApiGatewayProxyResponse {
        status_code: 200,
        headers: {
            let mut headers = HeaderMap::new();
            headers.insert("Content-Type", content_type.parse()?);
            headers.insert("Cache-Control", "public, max-age=60".parse()?);
            headers
        },
        body: Some(body),
        is_base64_encoded,
        ..Default::default()
    })
}
