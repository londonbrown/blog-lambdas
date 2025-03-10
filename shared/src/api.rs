use lambda_http::http::HeaderValue;
use lambda_http::{Response, Body};
use std::collections::HashMap;
use std::env;

/// Returns CORS headers if `ALLOWED_ORIGIN` is set; otherwise, rejects the request.
pub fn cors_headers() -> Result<HashMap<&'static str, HeaderValue>, Response<Body>> {
    let allowed_origin = env::var("ALLOWED_ORIGIN").map_err(|_| {
        Response::builder()
            .status(403)
            .body(Body::from("Forbidden: ALLOWED_ORIGIN not set"))
            .unwrap()
    })?;

    let mut headers = HashMap::new();
    headers.insert("Access-Control-Allow-Origin", HeaderValue::from_str(&allowed_origin).unwrap());
    headers.insert("Access-Control-Allow-Methods", HeaderValue::from_static("GET, POST, OPTIONS"));
    headers.insert("Access-Control-Allow-Headers", HeaderValue::from_static("Content-Type"));

    Ok(headers)
}

/// Adds CORS headers to a response, or returns `403 Forbidden` if `ALLOWED_ORIGIN` is missing.
pub fn with_cors<T>(response: Response<T>) -> Result<Response<T>, Response<Body>> {
    let headers = cors_headers()?;
    let mut response = response;
    for (key, value) in headers {
        response.headers_mut().insert(key, value);
    }
    Ok(response)
}