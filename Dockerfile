FROM rust:1.77-slim AS builder
WORKDIR /app

# Cache dependencies separately from source
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main(){}" > src/main.rs
RUN cargo build --release
RUN rm src/main.rs

# Build actual source
COPY src ./src
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update \
 && apt-get install -y ca-certificates \
 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/statsvg-rs /usr/local/bin/statsvg-rs
EXPOSE 3000
CMD ["statsvg-rs"]