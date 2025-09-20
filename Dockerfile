# 1️⃣ Build stage
FROM rust:1.89.0-slim-trixie AS builder

# Install necessary tools
RUN apt-get update && apt-get install -y pkg-config libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /usr/src/app

# Copy manifest files first (for caching dependencies)
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY .sqlx ./.sqlx
COPY migrations ./migrations

# Build release binary
RUN cargo build --release

# 2️⃣ Minimal distroless runtime stage
FROM gcr.io/distroless/cc

# Set working directory
WORKDIR /app

# Copy the statically compiled binary from builder
COPY --from=builder /usr/src/app/target/release/trmnl-server .

# Expose port
EXPOSE 3000

# Run the binary
CMD ["./trmnl-server"]
