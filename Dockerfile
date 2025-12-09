# https://dev.to/rogertorres/first-steps-with-docker-rust-30oi

# Rust nightly as the base image
FROM rust:1.91 AS build

# Create a new empty shell project
RUN USER=root cargo new --bin text-rpg
WORKDIR /text-rpg

# Copy our manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Build only the dependencies to cache them
# NOTE: text-rpg becomes text_rpg for deps
RUN cargo build --release && rm -rf src && rm -f ./target/release/deps/text_rpg* && rm -f ./target/release/text-rpg*

# Copy the source code
COPY ./src ./src

# Build for release.
RUN cargo build --release

# The final base image
FROM debian:bookworm-slim

# Install some deps
RUN apt-get update && apt-get -y install sqlite3 openssl ca-certificates

# Copy from the previous build
COPY --from=build /text-rpg/target/release/text-rpg /usr/src/text-rpg

ENV PATH="/usr/src/:$PATH"

RUN ls -la /usr/src/ && echo "$PATH"
# COPY --from=build /text-rpg/target/release/text-rpg/target/x86_64-unknown-linux-musl/release/text-rpg .

# Run the binary
CMD ["/usr/src/text-rpg"]