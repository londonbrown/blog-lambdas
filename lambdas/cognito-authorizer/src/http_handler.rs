use std::collections::HashMap;
use std::env;
use lambda_http::{Body, Error, Request, Response};
// use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
// use reqwest::Client;
use serde_json::Value;
use serde::{Deserialize, Serialize};
use tracing::log::info;

/// Structure of Cognito JWT Claims
#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    #[serde(rename = "cognito:groups")]
    groups: Option<Vec<String>>,
}

/// API Gateway Authorizer Response
#[derive(Debug, Serialize)]
struct AuthPolicy {
    principal_id: String,
    policy_document: PolicyDocument,
}

/// IAM Policy
#[derive(Debug, Serialize)]
struct PolicyDocument {
    version: String,
    statement: Vec<Statement>,
}

/// Policy Statement
#[derive(Debug, Serialize)]
struct Statement {
    effect: String,
    action: String,
    resource: String,
}

// async fn fetch_jwks(jwks_url: &str) -> Result<HashMap<String, DecodingKey>, Error> {
//     let client = Client::new();
//     let jwks: Value = client.get(jwks_url).send().await?.json().await?;
//     let mut keys = HashMap::new();
//
//     if let Some(keys_array) = jwks.get("keys").and_then(|k| k.as_array()) {
//         for key in keys_array {
//             if let (Some(kid), Some(n), Some(e)) = (
//                 key.get("kid").and_then(|v| v.as_str()),
//                 key.get("n").and_then(|v| v.as_str()),
//                 key.get("e").and_then(|v|v.as_str())
//             ) {
//                 let decoding_key = DecodingKey::from_rsa_components(n, e)?;
//                 keys.insert(kid.to_string(), decoding_key);
//             }
//         }
//     }
//     Ok(keys)
// }

pub(crate) async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    info!("Received event: {:?}", event);

    return Ok(Response::new(Body::Text("Done".to_string())));

    // let token = event
    //     .headers()
    //     .get("Authentication")
    //     .and_then(|v| v.to_str().ok())
    //     .and_then(|v| v.strip_prefix("Bearer "))
    //     .ok_or("Missing or invalid Authorization header")?;
    //
    // let user_pool_id = env::var("USER_POOL_ID").expect("USER_POOL_ID not set");
    //
    // let jwks_url = format!("https://cognito-idp.us-east-1.amazonaws.com/{}/.well-known.jwks.json", user_pool_id);
    // let jwks = fetch_jwks(&jwks_url).await?;
    //
    // let header = jsonwebtoken::decode_header(token)?;
    // let kid = header.kid.ok_or("Missing kid in token")?;
    // let decoding_key = jwks.get(&kid).ok_or("Invalid kid")?;
    //
    // let validation = Validation::new(Algorithm::RS256);
    // let token_data = decode::<Claims>(token, decoding_key, &validation)?;
    //
    // let claims = token_data.claims;
    // let is_admin = claims
    //     .groups
    //     .unwrap_or_default()
    //     .iter()
    //     .any(|g| g == "Admin");
    //
    // if is_admin {
    //     let policy = AuthPolicy {
    //         principal_id: claims.sub,
    //         policy_document: PolicyDocument {
    //             version: "2012-10-17".to_string(),
    //             statement: vec![Statement {
    //                 effect: "Allow".to_string(),
    //                 action: "execute-api:Invoke".to_string(),
    //                 resource: "*".to_string() // TODO
    //             }]
    //         }
    //     };
    //     Ok(Response::new(Body::Text(serde_json::to_string(&policy)?)))
    // } else {
    //     let policy = AuthPolicy {
    //         principal_id: claims.sub,
    //         policy_document: PolicyDocument {
    //             version: "2012-10-17".to_string(),
    //             statement: vec![Statement {
    //                 effect: "Deny".to_string(),
    //                 action: "execute-api:Invoke".to_string(),
    //                 resource: "*".to_string()
    //             }]
    //         }
    //     };
    //     Ok(Response::new(Body::Text(serde_json::to_string(&policy)?)))
    // }
}
