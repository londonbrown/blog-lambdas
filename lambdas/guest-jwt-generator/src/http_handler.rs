use std::collections::HashMap;
use std::env;
use aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use aws_lambda_events::encodings::Body;
use aws_lambda_events::http::HeaderMap;
use aws_lambda_events::http::header::CONTENT_TYPE;
use aws_sdk_cognitoidentityprovider::types::AuthFlowType;
use chrono::Utc;
use lambda_runtime::{LambdaEvent};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::OnceCell;
use tracing::info;


#[derive(Debug, Deserialize, Serialize)]
struct Tokens {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "idToken")]
    id_token: String,
    #[serde(rename = "refreshToken")]
    refresh_token: String,
    #[serde(rename = "expiresIn")]
    expires_in: i32,
    #[serde(rename = "tokenType")]
    token_type: String
}

static CACHED_TOKEN: OnceCell<(String, i64)> = OnceCell::const_new();
const CACHE_TTL: i64 = 900; // 15 minutes

async fn get_cached_token() -> Option<Tokens> {
    if let Some((cached_tokens, exp)) = CACHED_TOKEN.get() {
        if *exp > Utc::now().timestamp() {
            let tokens = serde_json::from_str(&cached_tokens.clone()).unwrap();
            return Some(tokens);
        }
    }
    None
}

async fn store_tokens(tokens_json: String) {
    let exp = Utc::now().timestamp() + CACHE_TTL;
    let _ = CACHED_TOKEN.set((tokens_json, exp));
}

pub(crate) async fn function_handler(event: LambdaEvent<ApiGatewayProxyRequest>) -> Result<ApiGatewayProxyResponse, Box<dyn std::error::Error>> {
    info!("event: {:?}", event);

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

    if let Some(cached_token) = get_cached_token().await {
        info!("Using cached token");
        return Ok(ApiGatewayProxyResponse {
            status_code: 200,
            headers,
            body: Some(Body::Text(json!({"token": cached_token}).to_string())),
            ..Default::default()
        });
    }
    info!("No cached token found");

    let config = aws_config::load_from_env().await;

    let cognito_password_secret_name = env::var("COGNITO_USER_PASSWORD_SECRET_NAME").expect("COGNITO_USER_PASSWORD_SECRET_NAME not set");
    let cognito_client_id = env::var("COGNITO_CLIENT_ID").expect("COGNITO_CLIENT_ID not set");

    let secrets_manager_client = aws_sdk_secretsmanager::Client::new(&config);

    let secret_string = secrets_manager_client.get_secret_value()
        .secret_id(cognito_password_secret_name)
        .send()
        .await?
        .secret_string
        .expect("secret_string not set");

    let username_password_map = serde_json::from_str::<HashMap<String, String>>(&secret_string)?;

    let username = username_password_map
        .get("username")
        .expect("username not set")
        .to_string();

    let password = username_password_map
        .get("password")
        .expect("password not set")
        .to_string();

    let auth_params = HashMap::from([("USERNAME".to_string(), username), ("PASSWORD".to_string(), password)]);
    let cognito_idp_client = aws_sdk_cognitoidentityprovider::Client::new(&config);

    let idp_response = cognito_idp_client.initiate_auth()
        .client_id(cognito_client_id)
        .auth_flow(AuthFlowType::UserPasswordAuth)
        .set_auth_parameters(Some(auth_params))
        .send()
        .await?;

    let authentication_result = idp_response.authentication_result.expect("authentication result not set");
    let tokens = Tokens {
        token_type: authentication_result.token_type.unwrap(),
        access_token: authentication_result.access_token.unwrap(),
        refresh_token: authentication_result.refresh_token.unwrap(),
        id_token: authentication_result.id_token.unwrap(),
        expires_in: authentication_result.expires_in,
    };


    let tokens_json = serde_json::to_string(&tokens).unwrap();
    store_tokens(tokens_json.clone()).await;

    Ok(ApiGatewayProxyResponse {
        status_code: 200,
        body: Some(Body::Text(tokens_json)),
        ..Default::default()
    })
}
