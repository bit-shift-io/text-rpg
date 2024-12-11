# https://dev.to/rogertorres/first-steps-with-docker-rust-30oi

# Rust nightly as the base image
FROM rustlang/rust:nightly AS build

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

# Copy from the previous build
COPY --from=build /text-rpg/target/release/text-rpg /usr/src/text-rpg
# COPY --from=build /text-rpg/target/release/text-rpg/target/x86_64-unknown-linux-musl/release/text-rpg .

# Run the binary
CMD ["/usr/src/text-rpg"]