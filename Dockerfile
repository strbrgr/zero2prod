# Builder stage
FROM rust:1.83.0 AS builder
WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .

# Handle the issue that sqlx runs queries
ENV SQLX_OFFLINE=true
RUN cargo build --release

#
# The builder stage does not contribute to its size, it is discarded
#

# Runtime stage
FROM debian:bookworm-slim AS runtime
WORKDIR /app
# OpenSSL & ca-certificates
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates \
  # Clean up
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/zero2prod zero2prod
# We need the configuration file at runtime~
COPY configuration configuration
ENV APP_ENVIRONMENT=production
ENTRYPOINT ["./zero2prod"]
