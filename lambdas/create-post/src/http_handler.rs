use chrono::Utc;
use lambda_http::{Body, Request, RequestExt, Response};
use serde_json::to_string;
use tracing::info;
use shared::models::{BlogPost, PostRequest};
use uuid::Uuid;

pub(crate) async fn function_handler(event: Request) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
    let request_context = event.request_context();
    info!("Full Request Context: {:?}", request_context);

    let authorizer = request_context.authorizer().ok_or("Missing authorizer context")?;
    info!("Authorizer: {:?}", authorizer);

    let jwt = authorizer.jwt.as_ref().ok_or("Missing JWT in authorizer")?;
    info!("JWT Claims: {:?}", jwt.claims);

    let claims = &jwt.claims;

    let author_id = claims.get("sub").ok_or("Missing sub claim")?.as_str();

    let body = event.body();
    let body_str = std::str::from_utf8(body)?;

    let post_request: PostRequest = serde_json::from_str(body_str)?;

    let post_id = format!("post-{}", Uuid::new_v4());
    let created_at = Utc::now().to_rfc3339();
    let content_key = Uuid::new_v4().to_string(); // ✅ Store content separately in S3

    let blog_post = BlogPost {
        pk: format!("POST#{}", post_id),
        sk: "META".to_string(),
        title: post_request.title,
        author_id: author_id.to_string(),
        tags: post_request.tags,
        published: post_request.published,
        created_at,
        content_key,
    };

    Ok(Response::builder()
        .status(200)
        .body(Body::Text(to_string(&blog_post)?)).unwrap())
}
