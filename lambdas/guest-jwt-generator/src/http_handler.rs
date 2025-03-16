use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse, ApiGatewayV2httpRequest};
use lambda_http::{Body, LambdaEvent};
use tracing::info;

pub(crate) async fn function_handler(event: LambdaEvent<ApiGatewayProxyRequest>) -> Result<ApiGatewayProxyResponse, Box<dyn std::error::Error>> {
    info!("event: {:?}", event);
    Ok(ApiGatewayProxyResponse {
        status_code: 200,
        body: Some(Body::Text("Hello, world!".to_string())),
        ..Default::default()
    })
}
