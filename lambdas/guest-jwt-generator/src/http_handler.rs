use aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use aws_lambda_events::encodings::Body;
use lambda_runtime::{LambdaEvent};
use tracing::info;

pub(crate) async fn function_handler(event: LambdaEvent<ApiGatewayProxyRequest>) -> Result<ApiGatewayProxyResponse, Box<dyn std::error::Error>> {
    info!("event: {:?}", event);
    Ok(ApiGatewayProxyResponse {
        status_code: 200,
        body: Some(Body::Text("Hello, world!".to_string())),
        ..Default::default()
    })
}
