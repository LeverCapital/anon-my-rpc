use axum::http::{Response, StatusCode};
use axum::{routing::post, Router};
use hyper::body::Bytes;
use hyper::Body;
use jsonrpc_core::Request;
use log::info;
use regex::Regex;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    env_logger::init();
    // Define router
    let app = Router::new().route("/*url", post(tor_proxy));
    info!("Starting server");

    // Run the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn tor_proxy(
    params: axum::extract::Path<String>,
    body: Bytes,
) -> Result<Response<Body>, StatusCode> {
    info!("Received request");

    // Extract the URL path
    let path = &params.0;

    // Create a regular expression to match the desired URL pattern
    let pattern = ".*\\.g\\.alchemy\\.com/.*";
    let regex = Regex::new(pattern).unwrap();

    // Check if the URL matches the desired pattern
    if !regex.is_match(path) {
        info!("{}", path);
        // Return an error response for disallowed URLs
        return Err(StatusCode::BAD_REQUEST);
    }

    let bytes = body.as_ref();
    let parsed_json: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    let method_name = parsed_json.get("method").and_then(|v| v.as_str());
    info!("method_name: {:?}", method_name);

    let request: Request = match serde_json::from_value(parsed_json) {
        Ok(request) => request,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    // Make sure you are running tor and this is your socks port
    let proxy = reqwest::Proxy::all("socks5://127.0.0.1:9150").expect("tor proxy should be there");
    let client = reqwest::Client::builder()
        .proxy(proxy)
        .build()
        .expect("should be able to build reqwest client");

    // Add your actual Infura and Alchemy URLs here
    let target_url = format!("https://{}", *params);
    info!("target_url: {}", target_url);

    // Forward the request
    let res = client
        .post(&target_url)
        .json(&request)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let axum_response = convert_reqwest_response_to_axum_response(res).await?;
    Ok(axum_response)
}

async fn convert_reqwest_response_to_axum_response(
    res: reqwest::Response,
) -> Result<Response<Body>, StatusCode> {
    let status_code = res.status().as_u16();
    let bytes = res
        .text()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let body = Body::from(bytes);
    let mut response = Response::new(body);
    *response.status_mut() = StatusCode::from_u16(status_code).unwrap();
    Ok(response)
}
