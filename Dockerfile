FROM rust:1.92.0 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM ubuntu:24.04
WORKDIR /app 
COPY --from=builder /app/target/release/match-engine /usr/local/bin/match-engine
CMD ["match-engine"]
