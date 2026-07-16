# Stage 1: Build
FROM rust:slim-bookworm as builder

WORKDIR /app

# [LOG] Beginning environment initialization
RUN echo "=== STAGE 1: Starting Rust Compilation Environment Setup ==="

# Install system dependencies needed for compilation
RUN apt-get update && apt-get install -y pkg-config libssl-dev curl && rm -rf /var/lib/apt/lists/* \
    && echo "✅ System packages (OpenSSL, pkg-config) installed successfully."

# Copy only Cargo.toml and Cargo.lock first to cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN echo "📥 Manifest files copied. Checking directory contents:" && ls -la

# Copy the migrations directory so sqlx::migrate! can find it at compile time
COPY migrations ./migrations
RUN echo "🗂️ Migrations directory copied for SQLx compile-time macro checks."

# Create a dummy main.rs and lib.rs to build dependencies and leverage Docker caching
RUN mkdir src && touch src/lib.rs && echo "fn main() {}" > src/main.rs \
    && echo "⚡ Building dependency cache layer..." \
    && cargo build --release \
    && echo "✅ Dependency cache built successfully." \
    && rm -rf src

# Copy the actual application source code
COPY src ./src
RUN echo "📥 Source code injected into workspace. Compiling final release binary..."

# Build the final release application binary
RUN cargo build --release \
    && echo "🎉 Binary compilation complete. Target production file verified:" \
    && ls -la target/release/lms_api

# Stage 2: Runtime (Minimal image for production)
FROM debian:bookworm-slim

WORKDIR /app

RUN echo "=== STAGE 2: Constructing Minimal Production Runtime ==="

# Install runtime dependencies 
# Using libssl-dev to prevent architecture mismatch crashes with sqlx/actix-web
RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/* \
    && echo "✅ Runtime libraries (OpenSSL dev, SSL Certificates) prepared."

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/lms_api /app/lms_api
RUN echo "🚀 Extracted compiled lms_api binary into runtime workspace." \
    && chmod +x /app/lms_api

# Expose the internal port (Render overrides this with the PORT env)
EXPOSE 8080

# Production environment configurations
ENV RUST_BACKTRACE=1
ENV RUST_LOG=debug
ENV RUST_LOG_STYLE=always

RUN echo "📡 Production runtime configuration set: RUST_BACKTRACE=1, RUST_LOG=debug"
RUN echo "🏁 Starting application execution stream..."

# Run the application
CMD ["/app/lms_api"]