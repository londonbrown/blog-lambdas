use aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use aws_lambda_events::encodings::Body;
use lambda_runtime::LambdaEvent;
use tracing::log::info;

pub(crate) async fn function_handler(
    event: LambdaEvent<ApiGatewayProxyRequest>,
) -> Result<ApiGatewayProxyResponse, Box<dyn std::error::Error + Send + Sync>> {
    let request_context = event.payload.request_context;
    let authorizer = request_context.authorizer;

    info!("Authorizer: {:?}", authorizer);

    let claims = authorizer
        .fields
        .get("claims")
        .ok_or("Missing claims in fields")?
        .as_object()
        .ok_or("Claims is not object")?;

    info!("Claims: {:?}", claims);

    // TODO extract group id from claims

    Ok(ApiGatewayProxyResponse {
        status_code: 200,
        body: Some(Body::Text("Hello, world!".to_string())),
        ..Default::default()
    })
}
