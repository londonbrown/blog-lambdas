use aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use aws_lambda_events::encodings::Body;
use aws_sdk_dynamodb::Client;
use chrono::Utc;
use lambda_runtime::LambdaEvent;
use shared::api::get_author_id_from_request_context;
use shared::db::create_post;
use shared::models::{BlogPost, PostRequest};
use std::env;
use tracing::info;
use uuid::Uuid;

pub(crate) async fn function_handler(
    event: LambdaEvent<ApiGatewayProxyRequest>,
) -> Result<ApiGatewayProxyResponse, Box<dyn std::error::Error + Send + Sync>> {
    let request = event.payload;

    let request_context = request.request_context;
    let body = request.body.ok_or("Missing body")?;

    let author_id = get_author_id_from_request_context(request_context)?;

    let post_request: PostRequest = serde_json::from_str(&body)?;

    info!("Post request: {:#?}", post_request);

    let post_id = format!("post-{}", Uuid::new_v4());
    let created_at = Utc::now().to_rfc3339();
    let content_key = Uuid::new_v4().to_string(); // ✅ Store content separately in S3
    let post_pk = format!("POST#{}", post_id);

    let client = Client::new(&aws_config::load_from_env().await);
    let table_name = env::var("BLOG_POSTS_TABLE").expect("BLOG_POSTS_TABLE not set");

    let blog_post = BlogPost {
        pk: post_pk.clone(),
        sk: "META".to_string(),
        title: post_request.title,
        author_id: author_id.to_string(),
        tags: post_request.tags,
        published: post_request.published,
        created_at,
        content_key,
    };

    info!("Blog post: {:?}", blog_post);

    match create_post(&client, &table_name, &blog_post).await {
        Ok(_) => Ok(ApiGatewayProxyResponse {
            status_code: 201,
            body: Some(Body::Text(format!("{{\"post_id\": \"{}\"}}", post_pk))),
            ..Default::default()
        }),
        Err(err) => Ok(ApiGatewayProxyResponse {
            status_code: 409,
            body: Some(Body::Text(format!("{{\"error\": \"{}\"}}", err))),
            ..Default::default()
        }),
    }
}
