#FROM debian:bookworm-slim
FROM rust:1.82-slim-bullseye

# Install runtime dependencies
#RUN apt-get update -y && \
#    apt-get install -y libpq-dev libssl3 ca-certificates file && \
#    rm -rf /var/lib/apt/lists/*
#

RUN apt-get update -y && \
    apt-get upgrade -y && \
    apt-get install -y pkg-config libssl-dev libpq-dev curl

WORKDIR /app

COPY ./Cargo.toml Cargo.toml
COPY ./migration-script.sh .
COPY ./schema-up.sql .
COPY ./schema-down.sql .

# Copy compiled binary with verification
COPY ./target/release/morning_compass_api .
COPY ./api-response.json .
COPY ./wait-for-it.sh .

RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo install cargo-watch

CMD ["./morning_compass_api"]
