ARG RUST_VERSION=1.82.0

# Setup
FROM rust:${RUST_VERSION} as setup
WORKDIR /app
COPY Cargo.toml ./

# Build
FROM setup as build
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo fetch
RUN cargo build --release
RUN rm src/main.rs
COPY src ./src/
RUN touch src/main.rs
RUN cargo build --release

# dev image
FROM setup as dev
COPY src ./src/
RUN cargo build
ENTRYPOINT [ "cargo" ]

# live image
FROM debian:stable-slim as live
RUN apt-get update && \
  apt-get install -y ca-certificates openssl && \
  apt-get clean
RUN apt-get autoremove
WORKDIR /app
COPY --from=build /app/target/release/captsone-rust exec
ENTRYPOINT [ "./exec" ]

