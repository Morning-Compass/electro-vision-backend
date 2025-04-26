FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update -y && \
    apt-get install -y libpq-dev libssl3 ca-certificates file && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy compiled binary with verification
COPY ./target/release/morning_compass_api .

CMD ["./morning_compass_api"]
