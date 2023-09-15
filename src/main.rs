use arti_client::{TorClient, TorClientConfig};
use arti_hyper::ArtiHttpConnector;
use axum::http::{Response, StatusCode};
use axum::{
    extract::{Json, Path, State},
    routing::post,
    Router,
};
use hyper::{header::CONTENT_TYPE, Body, Method, Request as HyperRequest};
use jsonrpc_core::Request as RpcRequest;
use log::info;
use regex::RegexSet;
use std::net::SocketAddr;
use tls_api::{TlsConnector as TlsConnectorTrait, TlsConnectorBuilder};
use tls_api_native_tls::TlsConnector;
use tor_rtcompat::PreferredRuntime;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Initialise Tor client
    let config = TorClientConfig::default();
    info!("Bootstrapping connection to the Tor network...");
    let tor_client = TorClient::create_bootstrapped(config).await?;

    // Define router
    let app = Router::new()
        .route("/*url", post(proxy))
        .with_state(tor_client);

    info!("Starting server...");
    // Run the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}

async fn proxy(
    Path(url): Path<String>,
    State(tor_client): State<TorClient<PreferredRuntime>>,
    Json(payload): Json<RpcRequest>,
) -> Result<Response<Body>, StatusCode> {
    info!("Received request");

    //TODO: Remove any debug prints
    println!("{}", url);

    // Create a RegexSet to match against popular Node providers
    let patterns = RegexSet::new(&[
        ".*\\.g\\.alchemy\\.com/.*", // Alchemy
        ".*\\.infura\\.io\\/v3\\/.*" // Infura v3
    ]).unwrap();

    // Check if the request URL matches the desired pattern
    if !patterns.is_match(url.as_str()) {
        info!("{}", url);
        // Return an error response for disallowed URLs
        return Err(StatusCode::BAD_REQUEST);
    }

    info!("Forwarding request to Tor...");

    // INFO: Using the same Tor circuit for all forwarded requests right now.
    // Spawning a new circuit per request slows down the proxy. Maybe spawn one per N requests?
    // let isolated_tor_client = tor_client.isolated_client();
    let response = match forward_through_tor(url, payload, tor_client).await {
        Ok(response) => response,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    Ok(response)
}

async fn forward_through_tor(
    url: String,
    payload: RpcRequest,
    tor_client: TorClient<PreferredRuntime>,
) -> Result<Response<Body>, Box<dyn std::error::Error>> {
    // Build the request
    let target_url = format!("https://{}", url);
    let req = HyperRequest::builder()
        .method(Method::POST)
        .uri(target_url)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&payload)?))?;

    let tls_connector = TlsConnector::builder()?.build()?;
    let tor_connector = ArtiHttpConnector::new(tor_client, tls_connector);
    let tor_http = hyper::Client::builder().build::<_, Body>(tor_connector);

    let resp = tor_http.request(req).await?;
    Ok(resp)
}
