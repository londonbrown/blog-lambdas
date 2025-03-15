use std::env;
use aws_sdk_dynamodb::Client;
use chrono::Utc;
use lambda_http::{Body, Request, RequestExt, Response};
use tracing::info;
use shared::models::{BlogPost, PostRequest};
use uuid::Uuid;
use shared::db::create_post;

pub(crate) async fn function_handler(event: Request) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
    let request_context = event.request_context();
    info!("Full Request Context: {:?}", request_context);

    let authorizer = request_context.authorizer().ok_or("Missing authorizer context")?;
    info!("Authorizer: {:?}", authorizer);

    let author_id = authorizer
        .fields
        .get("user")
        .map(|v| v.to_string())
        .ok_or("Missing user in fields")?;

    let body = event.body();
    let body_str = std::str::from_utf8(body)?;

    let post_request: PostRequest = serde_json::from_str(body_str)?;

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


    match create_post(&client, &table_name, &blog_post).await {
        Ok(_) => Ok(Response::builder()
            .status(201)
            .body(Body::Text(format!("{{\"post_id\": \"{}\"}}", post_pk)))
            .unwrap()),
        Err(err) => Ok(Response::builder()
            .status(409) // 409 Conflict
            .body(Body::Text(format!("{{\"error\": \"{}\"}}", err)))
            .unwrap()),
    }
}
