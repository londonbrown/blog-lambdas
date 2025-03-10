use lambda_http::{Body, Error, Request, Response};
use serde_json::{json};

pub(crate) async fn function_handler(_event: Request) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(404)
        .body(json!({"error": "Post not found"}).to_string().into())?)
}
