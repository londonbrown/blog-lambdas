use aws_lambda_events::apigw::ApiGatewayProxyRequestContext;
use std::error::Error;

pub fn get_author_id_from_request_context(
    request_context: ApiGatewayProxyRequestContext,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let fields = request_context.authorizer.fields;

    let claims = fields.get("claims").ok_or("Missing claims in fields");

    let author_id = claims?
        .get("sub")
        .ok_or("Missing sub in claims")?
        .as_str()
        .ok_or("sub is not defined")?;

    Ok(author_id.to_string())
}
