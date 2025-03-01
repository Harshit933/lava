FROM rust:latest as builder

WORKDIR /usr/src/app

# Install dependencies needed for compilation
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    openssl \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Create a dummy src/main.rs to build dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub mod error { pub enum LavaErrors {} }" > src/error.rs

# Build dependencies
RUN cargo build --release

# Remove the dummy source files
RUN rm -rf src

# Copy the actual source code
COPY src ./src

# Rebuild with the actual source
RUN touch src/main.rs src/error.rs && cargo build --release

# Create a new stage with a minimal image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    curl \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/lava /app/app

# Download the loans-borrower-cli
RUN curl -o loans-borrower-cli https://loans-borrower-cli.s3.amazonaws.com/loans-borrower-cli-linux && \
    chmod +x loans-borrower-cli

# Copy the .env file if you have one
COPY .env .

# Expose port 3000
EXPOSE 3000

# Run the binary
CMD ["/app/app"]