use lambda_http::{Body, Request, RequestExt, Response};
use tracing::log::info;

pub(crate) async fn function_handler(event: Request) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
    let request_context = event.request_context();
    let authorizer = request_context.authorizer().ok_or("Missing authorizer context")?;

    info!("Authorizer: {:?}", authorizer);

    let claims = authorizer.fields.get("claims").ok_or("Missing claims in fields")?.as_object().ok_or("Claims is not object")?;

    info!("Claims: {:?}", claims);

    // TODO extract group id from claims

    Ok(Response::builder()
        .status(200)
        .body("Hello, World!".into()).unwrap())
}
