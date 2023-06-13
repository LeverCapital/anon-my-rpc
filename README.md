# Anon My RPC

![anonmyrpc-demo](https://github.com/LeverCapital/anon-my-rpc/assets/2914233/7098c893-99cd-468d-90bf-2d33801cf74c)

## Overview

Anon My RPC is an RPC forwarding service that operates over the Tor network. It allows for secure and anonymous RPC calls to your favourite RPC providers like Alchemy, POKT, Infura etc. It achieves this by routing calls through Tor's anonymising circuits.

## Features

- Prefix based routing for RPC requests.
- Bootstrapped Tor client to route requests. No need to run your own.
- JSON RPC request validation and forwarding.
- Supports calls to Alchemy. (POKT and Infura coming soon)

## Usage

Once you have cloned this repository and installed Rust, you can compile and run the project using `cargo`.

```
$ cargo run
```

This will start the server on port `3000` by default.

## Dependencies

This project makes use of the following dependencies:

- `arti-client`: Used for setting up the Tor client.
- `arti-hyper`: An HTTP connector that works over the Tor client.
- `axum`: For defining the HTTP server and its routes.
- `hyper`: For creating and executing HTTP requests.
- `jsonrpc_core`: For validating JSON RPC requests.
- `regex`: For matching the request URL with the desired pattern.
- `tls_api` and `tls_api_native_tls`: For setting up TLS connections.
- `tor_rtcompat`: Provides a runtime for the Tor client.

## Error Handling

The project returns HTTP status codes to signify specific errors:

- `400 BAD REQUEST`: Returned if the URL does not match the desired pattern or if the payload is not a valid JSON RPC request.
- `500 INTERNAL SERVER ERROR`: Returned if there's any error during the execution of the request.

## Contributing

We welcome contributions to Anon My RPC! Please feel free to open an issue or pull request if you find a bug or see potential improvements.

## License

Anon My RPC is licensed under the [MIT license](LICENSE).
