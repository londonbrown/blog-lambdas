use crate::models::Claims;
use aws_lambda_events::apigw::ApiGatewayProxyRequestContext;
use serde_json::from_value;
use std::error::Error;

pub fn get_author_id_from_request_context(
    request_context: ApiGatewayProxyRequestContext,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let fields = request_context.authorizer.fields;

    let claims: Claims = match fields.get("claims") {
        Some(value) => from_value::<Claims>(value.clone()).unwrap_or_default(),
        None => Claims::default(),
    };

    let author_id = claims.sub;

    Ok(author_id.to_string())
}
