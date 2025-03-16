use std::collections::{HashMap, HashSet};
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
            format!("{}/admin.read", api_blog_domain_url),
            format!("{}/admin.write", api_blog_domain_url),
            format!("{}/admin.delete", api_blog_domain_url)
        ]),
        ("author", vec![
            format!("{}/author.read", api_blog_domain_url),
            format!("{}/author.write", api_blog_domain_url),
            format!("{}/author.delete", api_blog_domain_url)
        ]),
        ("commenter", vec![
            format!("{}/commenter.read", api_blog_domain_url),
            format!("{}/commenter.write", api_blog_domain_url),
            format!("{}/commenter.delete", api_blog_domain_url)
        ]),
        ("guest", vec![
            format!("{}/guest.read", api_blog_domain_url),
            format!("{}/guest.write", api_blog_domain_url),
            format!("{}/guest.delete", api_blog_domain_url)
        ]),
    ]);

    let mut assigned_scopes: HashSet<String> = HashSet::new();
    assigned_scopes.insert("aws.cognito.signin.user.admin".to_string());

    for group in request.group_configuration.groups_to_override.clone() {
        if let Some(scopes) = group_to_scope_map.get(group.as_str()) {
            assigned_scopes.extend(scopes.iter().cloned())
        }
    }

    let requested_scopes: HashSet<String> = request.scopes.iter().cloned().collect();
    let suppressed_scopes: Vec<String> = requested_scopes.difference(&assigned_scopes).cloned().collect();

    let response = CognitoEventUserPoolsPreTokenGenResponseV2 {
        claims_and_scope_override_details: Some(ClaimsAndScopeOverrideDetailsV2 {
            access_token_generation: Some(CognitoAccessTokenGenerationV2 {
                scopes_to_add: assigned_scopes.into_iter().collect(),
                scopes_to_suppress: suppressed_scopes,
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
