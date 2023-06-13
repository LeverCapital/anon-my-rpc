FROM rust:1.69.0
RUN apt-get update && apt-get install -y sqlite3 sqlite3-doc openssl

COPY . .
RUN cargo install --path .

EXPOSE 3000
ENV RUST_LOG=info
CMD ["anon_rpc"]
