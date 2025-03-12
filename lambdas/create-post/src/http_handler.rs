use lambda_http::{Body, Request, Response};

// TODO implement later
pub(crate) async fn function_handler(_event: Request) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Response::builder()
        .status(200)
        .body(Body::from("Hello, World!"))
        .unwrap())
}
