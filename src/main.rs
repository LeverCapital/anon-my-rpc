use axum::http::StatusCode;
use axum::response::Json;
use axum::{routing::post, Router};
use hyper::body::Bytes;
use jsonrpc_core::{Request, Response};
use log::{debug, error, info, warn};
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
) -> Result<Json<String>, StatusCode> {
    info!("Received request");
    let bytes = body.as_ref();
    let parsed_json: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    let method_name = match parsed_json.get("method").and_then(|v| v.as_str()) {
        Some("eth_sendRawTransaction")
        | Some("eth_estimateGas")
        | Some("eth_getTransactionCount")
        | Some("eth_getBlockByNumber") => {
            match parsed_json.get("method").unwrap().as_str().unwrap() {
                method => method,
            }
        }
        _ => return Err(StatusCode::FORBIDDEN),
    };

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
    let res = client.post(&target_url).json(&request).send().await;

    match res {
        Ok(res) => {
            // Parse the response as JSON-RPC
            let rpc_res: Result<Response, _> = res.json().await;
            info!("res: {:?}", rpc_res);
            match rpc_res {
                Ok(rpc_res) => {
                    // If the response is a JSON-RPC response, serialize it back to JSON and return
                    match serde_json::to_string(&rpc_res) {
                        Ok(json) => {
                            info!("json: {}", json);
                            Ok(Json(json))
                        }
                        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
                    }
                }
                Err(_) => {
                    info!("ffrfre");
                    // If the response couldn't be parsed as JSON-RPC, return a 502 Bad Gateway status code
                    Err(StatusCode::BAD_GATEWAY)
                }
            }
        }
        Err(_) => Err(StatusCode::BAD_GATEWAY),
    }
}
