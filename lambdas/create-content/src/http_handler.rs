use aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use aws_lambda_events::encodings::Body;
use aws_sdk_dynamodb::Client as DdbClient;
use aws_sdk_s3::Client as S3Client;
use base64::engine::general_purpose::STANDARD as base64_engine;
use base64::Engine as _;
use chrono::Utc;
use lambda_runtime::LambdaEvent;
use shared::api::get_author_id_from_request_context;
use shared::db::create_content;
use shared::models::{Content, CreateContentRequest, CreateContentResponse};
use std::env;
use tracing::info;
use uuid::Uuid;

pub(crate) async fn function_handler(
    event: LambdaEvent<ApiGatewayProxyRequest>,
) -> Result<ApiGatewayProxyResponse, Box<dyn std::error::Error + Send + Sync>> {
    let request = event.payload;
    let body = request.body.ok_or("Missing body")?;

    let request_context = request.request_context;

    info!("Request context: {:?}", request_context);

    let author_id = get_author_id_from_request_context(request_context)?;

    info!("author id: {:?}", author_id);

    let parsed: CreateContentRequest = serde_json::from_str(&body)?;

    info!("Create content request: {:#?}", parsed);

    let content_id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();

    // Upload content to S3
    let bucket = env::var("BLOG_CONTENT_BUCKET")?;
    let key = format!("content/{}.{}", content_id, parsed.file_extension);

    let s3 = S3Client::new(&aws_config::load_from_env().await);

    let data = if parsed.is_base64_encoded {
        base64_engine.decode(parsed.body.trim())?
    } else {
        parsed.body.into_bytes()
    };

    s3.put_object()
        .bucket(&bucket)
        .key(&key)
        .body(aws_sdk_s3::primitives::ByteStream::from(data))
        .content_type(parsed.content_type.clone())
        .send()
        .await?;

    // Write metadata to DynamoDB
    let content_pk = format!("CONTENT#{}", content_id);
    let location = format!("s3://{}/{}", bucket, key);

    let ddb = DdbClient::new(&aws_config::load_from_env().await);
    let table_name = env::var("BLOG_CONTENT_TABLE").expect("BLOG_CONTENT_TABLE not set");

    let content = Content {
        pk: content_pk.clone(),
        sk: "META".to_string(),
        content_type: parsed.content_type.clone(),
        location: location.to_string(),
        created_at,
        author_id,
    };

    match create_content(&ddb, &table_name, &content).await {
        Ok(_) => {
            let response = CreateContentResponse {
                content_id,
                content_type: content.content_type,
                location: content.location,
                created_at: content.created_at,
                author_id: content.author_id,
            };
            Ok(ApiGatewayProxyResponse {
                status_code: 201,
                body: Some(Body::Text(serde_json::to_string(&response)?)),
                ..Default::default()
            })
        }
        Err(err) => Ok(ApiGatewayProxyResponse {
            status_code: 409,
            body: Some(Body::Text(format!("{{\"error\": \"{}\"}}", err))),
            ..Default::default()
        }),
    }
}
