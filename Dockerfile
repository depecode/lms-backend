# Stage 1: Build
FROM rust:slim-bookworm as builder

WORKDIR /app

# Install system dependencies needed for compilation
RUN apt-get update && apt-get install -y pkg-config libssl-dev curl && rm -rf /var/lib/apt/lists/*

# Copy only Cargo.toml and Cargo.lock first to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Copy the migrations directory so sqlx::migrate! can find it at compile time
COPY migrations ./migrations

# Create a dummy main.rs and lib.rs to build dependencies and leverage Docker caching
RUN mkdir src && touch src/lib.rs && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

# Copy the actual application source code
COPY src ./src

# Build the final release application binary
RUN cargo build --release

# Stage 2: Runtime (Minimal image for production)
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies (like CA certificates for secure connections to Supabase)
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/lms_api /app/lms_api

# Expose the internal port (Render overrides this with the PORT env, but it is good practice)
EXPOSE 8080

# Run the application
CMD ["/app/lms_api"]