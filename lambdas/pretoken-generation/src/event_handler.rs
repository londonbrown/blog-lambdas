use std::collections::HashMap;
use std::env;
use aws_lambda_events::cognito::{ClaimsAndScopeOverrideDetailsV2, CognitoAccessTokenGenerationV2, CognitoEventUserPoolsPreTokenGenRequestV2, CognitoEventUserPoolsPreTokenGenResponseV2};
use lambda_runtime::LambdaEvent;
use tracing::info;

pub(crate) async fn function_handler(event: LambdaEvent<CognitoEventUserPoolsPreTokenGenRequestV2>) -> Result<CognitoEventUserPoolsPreTokenGenResponseV2, Box<dyn std::error::Error>> {
    let request = event.payload;
    info!("Pre Token Generation Triggered: {:?}", request);

    let api_blog_domain = env::var("API_BLOG_DOMAIN").expect("API_BLOG_DOMAIN is not set");
    let api_blog_domain_url = format!("https://{}/", api_blog_domain);

    let group_to_scope_map: HashMap<&str, Vec<String>> = HashMap::from([
        ("Admin", vec![
            format!("{}/post.read", api_blog_domain_url),
            format!("{}/post.write", api_blog_domain_url),
            format!("{}/post.delete", api_blog_domain_url),
            format!("{}/comment.read", api_blog_domain_url),
            format!("{}/comment.write", api_blog_domain_url),
            format!("{}/comment.delete", api_blog_domain_url),
        ]),
        ("Author", vec![
            format!("{}/post.read", api_blog_domain_url),
            format!("{}/post.write", api_blog_domain_url),
            format!("{}/comment.read", api_blog_domain_url),
            format!("{}/comment.write", api_blog_domain_url),
            format!("{}/comment.delete", api_blog_domain_url),
        ]),
        ("Commenter", vec![
            format!("{}/comment.read", api_blog_domain_url),
            format!("{}/comment.write", api_blog_domain_url),
            format!("{}/comment.delete", api_blog_domain_url),
        ]),
        ("Guest", vec![
            format!("{}/post.read", api_blog_domain_url),
            format!("{}/comment.read", api_blog_domain_url),
        ]),
    ]);

    let mut assigned_scopes = Vec::new();

    for group in request.group_configuration.groups_to_override {
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

    Ok(response)
}
