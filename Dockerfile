# Use the official Rust image as base
FROM rust:1.75

# Install required dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    openssl \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src/ ./src/

# Build the Phat contract
RUN cargo build --release

# Expose port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Run the Phat contract
CMD ["cargo", "run", "--release"]