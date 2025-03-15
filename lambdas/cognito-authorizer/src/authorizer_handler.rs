use std::collections::HashMap;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use reqwest;
use serde_json::Value;
use std::env;
use aws_lambda_events::apigw::{ApiGatewayCustomAuthorizerPolicy, ApiGatewayCustomAuthorizerRequestTypeRequest, ApiGatewayCustomAuthorizerResponse};
use aws_lambda_events::iam::{IamPolicyEffect, IamPolicyStatement};
use lambda_runtime::LambdaEvent;
use tracing::info;

pub(crate) async fn function_handler(event: LambdaEvent<ApiGatewayCustomAuthorizerRequestTypeRequest>) -> Result<ApiGatewayCustomAuthorizerResponse<Option<HashMap<String, Value>>>, Box<dyn std::error::Error>> {
    info!("Received event: {:?}", event);

    info!("Event payload: {:?}", event.payload);

    let auth_header = event.payload
        .headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or("Missing Authorization header")?;
    info!("Extracted Authorization header: {}", auth_header);

    let method = event.payload.http_method.expect("event.payload.http_method is undefined").to_string();
    let resource_path = event.payload.request_context.resource_path.expect("request_context.resource_path is not set");
    info!("Method: {}, Resource Path: {}", method, resource_path);

    let user_pool_id = env::var("USER_POOL_ID").expect("USER_POOL_ID is not set");
    let region = env::var("AWS_REGION").expect("AWS_REGION is not set");

    let jwks_url = format!(
        "https://cognito-idp.{}.amazonaws.com/{}/.well-known/jwks.json",
        region, user_pool_id
    );
    info!("Fetching JWKS from: {}", jwks_url);

    let jwks: Value = reqwest::get(&jwks_url).await?.json::<Value>().await?;
    let jwk_keys = jwks["keys"].as_array().ok_or("Invalid JWKS format")?;

    let header = decode_header(auth_header)?;
    let kid = header.kid.ok_or("Missing 'kid' in JWT header")?;
    info!("Extracted JWT 'kid': {}", kid);

    let jwk = jwk_keys
        .iter()
        .find(|jwk| jwk["kid"].as_str() == Some(&kid))
        .ok_or("No matching JWK found")?;

    let n = jwk["n"].as_str().ok_or("Missing 'n' in JWK")?;
    let e = jwk["e"].as_str().ok_or("Missing 'e' in JWK")?;

    info!("n: {}", n);
    info!("e: {}", e);

    let expected_audience = env::var("COGNITO_CLIENT_ID").expect("COGNITO_CLIENT_ID not set");
    let expected_issuer = format!("https://cognito-idp.{}.amazonaws.com/{}", region, user_pool_id);

    let decoding_key = DecodingKey::from_rsa_components(n, e)?;
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[expected_audience]);
    validation.set_issuer(&[expected_issuer]);

    let token_data = decode::<Value>(auth_header, &decoding_key, &validation)?;
    let claims = token_data.claims;

    info!("Decoded claims: {:?}", claims);

    let user_id = claims["sub"].as_str().unwrap_or("unknown-user");
    let groups = claims["cognito:groups"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
        .unwrap_or_default();

    info!("User groups: {:?}", groups);

    let is_admin = groups.contains(&"Admin");

    let effect = if is_admin || (method == "POST" && resource_path == "/post") {
        IamPolicyEffect::Allow
    } else {
        IamPolicyEffect::Deny
    };

    let policy = ApiGatewayCustomAuthorizerResponse {
        principal_id: Some(user_id.to_string()),

        policy_document: ApiGatewayCustomAuthorizerPolicy {
            version: Some("2012-10-17".to_string()),
            statement: vec![IamPolicyStatement {
                action: vec!["execute-api:Invoke".to_string()],
                effect,
                resource: vec![format!(
                    "arn:aws:execute-api:{}:{}:{}/*/{}",
                    region,
                    env::var("AWS_ACCOUNT_ID")?,
                    env::var("API_GATEWAY_ID")?,
                    method
                )],
                ..Default::default()
            }],
        },

        context: Some(HashMap::from([
            ("user".to_string(), Value::String(user_id.to_string()))
        ])),

        usage_identifier_key: None,
    };

    info!("Policy: {:?}", policy);

    Ok(policy)
}
