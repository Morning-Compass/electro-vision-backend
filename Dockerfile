FROM rust:1.82-slim-bullseye

# Install dependencies
RUN apt-get update -y && \
    apt-get upgrade -y && \
    apt-get install -y pkg-config libssl-dev libpq-dev curl

# Download and make wait-for-it.sh executable

# Set working directory
WORKDIR /app

# Copy files
COPY ./Cargo.toml Cargo.toml
COPY . .

# Install diesel_cli and cargo-watch
RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo install cargo-watch


# Default command
CMD ["./wait-for-it.sh", "db:5432", "--", "cargo", "run"]
