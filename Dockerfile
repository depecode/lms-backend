# Stage 1: Build
FROM rust:slim-bookworm as builder

WORKDIR /app

# Install standard dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy only Cargo.toml and Cargo.lock first to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Cache breaker: Force Render to install curl and ignore previous cargo build cache
RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Create a dummy main.rs and lib.rs to build dependencies
RUN mkdir src && touch src/lib.rs && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

# Copy the actual source code
COPY src ./src

# Build the application
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/lms_api /app/lms_api

EXPOSE 8080

CMD ["/app/lms_api"]