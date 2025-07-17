# syntax=docker/dockerfile:1.4

# Stage 1: Build with Nightly Rust
FROM rustlang/rust:nightly AS builder

WORKDIR /app

# Copy dependency files first to use Docker cache
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Copy the full source and build actual binary
COPY . .
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

# Copy only the built binary
COPY --from=builder /app/target/release/kv-store /usr/local/bin/kv-store

EXPOSE 3000
CMD ["kv-store"]
