# Stage 1: Build
FROM rust:slim-bookworm as builder

WORKDIR /app

# Install dependencies needed for compilation
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy only Cargo.toml and Cargo.lock first to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs and lib.rs to build dependencies
RUN mkdir src && touch src/lib.rs && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

# Copy the actual source code
COPY src ./src

# Build the application
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies (like CA certificates for SSL/TLS)
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/lms_api /app/lms_api

# Expose the port
EXPOSE 8080

# Run the application
CMD ["/app/lms_api"]
