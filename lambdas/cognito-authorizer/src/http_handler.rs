use lambda_http::{Body, Request, RequestExt, Response};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use reqwest;
use serde_json::{json, Value};
use std::env;
use tracing::info;

pub(crate) async fn function_handler(event: Request) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
    info!("Received event: {:?}", event);

    info!("Request context: {:?}", event.request_context());

    let auth_header = event
        .headers()
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or("Missing Authorization header")?;
    info!("Extracted Authorization header: {}", auth_header);

    let method = event.method().as_str();
    let path = event.uri().path();
    info!("Method: {}, Path: {}", method, path);

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

    let effect = if is_admin || (method == "POST" && path == "/post") {
        "Allow"
    } else {
        "Deny"
    };

    let policy = json!({
        "principalId": user_id,
        "policyDocument": {
            "Version": "2012-10-17",
            "Statement": [{
                "Action": "execute-api:Invoke",
                "Effect": effect,
                "Resource": format!(
                    "arn:aws:execute-api:{}:{}:{}/*/{}",
                    region,
                    env::var("AWS_ACCOUNT_ID")?,
                    env::var("API_GATEWAY_ID")?,
                    method
                )
            }]
        }
    });

    info!("Generated policy: {:?}", policy);

    Ok(Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(Body::Text(policy.to_string()))?)
}
