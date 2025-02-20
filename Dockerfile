# Use the official Rust image as the base
FROM rust:1.82-slim-bullseye

# Install necessary dependencies
RUN apt-get update -y && \
    apt-get upgrade -y && \
    apt-get install -y pkg-config libssl-dev libpq-dev curl

# Copy the wait-for-it.sh script into the image
COPY wait-for-it.sh /wait-for-it.sh
RUN chmod +x /wait-for-it.sh

# Set working directory
ENV CARGO_TARGET_DIR=/tmp/target
WORKDIR /app

# Copy Cargo.toml and other necessary files
COPY ./Cargo.toml Cargo.toml
COPY . .

# Install necessary Rust crates
RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo install cargo-watch

# Run your application with wait-for-it to ensure PostgreSQL is ready
CMD /wait-for-it.sh db:5432 -- cargo run
