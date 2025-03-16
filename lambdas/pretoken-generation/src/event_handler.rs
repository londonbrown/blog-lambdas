use std::collections::HashMap;
use std::env;
use aws_lambda_events::cognito::{ClaimsAndScopeOverrideDetailsV2, CognitoAccessTokenGenerationV2, CognitoEventUserPoolsPreTokenGenResponseV2, CognitoEventUserPoolsPreTokenGenV2};
use lambda_runtime::LambdaEvent;
use tracing::info;

pub(crate) async fn function_handler(event: LambdaEvent<CognitoEventUserPoolsPreTokenGenV2>) -> Result<CognitoEventUserPoolsPreTokenGenV2, Box<dyn std::error::Error>> {
    info!("Full event: {:?}", event);

    let request = event.payload.request;
    info!("Pre Token Generation Triggered: {:?}", request);

    let api_blog_domain = env::var("API_BLOG_DOMAIN").expect("API_BLOG_DOMAIN is not set");
    let api_blog_domain_url = format!("https://{}", api_blog_domain);

    let group_to_scope_map: HashMap<&str, Vec<String>> = HashMap::from([
        ("admin", vec![
            format!("{}/post.read", api_blog_domain_url),
            format!("{}/post.write", api_blog_domain_url),
            format!("{}/post.delete", api_blog_domain_url),
            format!("{}/comment.read", api_blog_domain_url),
            format!("{}/comment.write", api_blog_domain_url),
            format!("{}/comment.delete", api_blog_domain_url),
        ]),
        ("author", vec![
            format!("{}/post.read", api_blog_domain_url),
            format!("{}/post.write", api_blog_domain_url),
            format!("{}/comment.read", api_blog_domain_url),
            format!("{}/comment.write", api_blog_domain_url),
            format!("{}/comment.delete", api_blog_domain_url),
        ]),
        ("commenter", vec![
            format!("{}/comment.read", api_blog_domain_url),
            format!("{}/comment.write", api_blog_domain_url),
            format!("{}/comment.delete", api_blog_domain_url),
        ]),
        ("guest", vec![
            format!("{}/post.read", api_blog_domain_url),
            format!("{}/comment.read", api_blog_domain_url),
        ]),
    ]);

    let mut assigned_scopes = Vec::new();

    for group in request.group_configuration.groups_to_override.clone() {
        if let Some(scopes) = group_to_scope_map.get(group.as_str()) {
            assigned_scopes.extend_from_slice(scopes);
        }
    }

    let response = CognitoEventUserPoolsPreTokenGenResponseV2 {
        claims_and_scope_override_details: Some(ClaimsAndScopeOverrideDetailsV2 {
            access_token_generation: Some(CognitoAccessTokenGenerationV2 {
                scopes_to_add: assigned_scopes,
                ..Default::default()
            }),
            ..Default::default()
        })
    };

    info!("Modified Scopes: {:?}", response);

    Ok(CognitoEventUserPoolsPreTokenGenV2 {
        cognito_event_user_pools_header: event.payload.cognito_event_user_pools_header,
        request,
        response
    })
}
