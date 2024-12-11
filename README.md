# Text RPG

Install AIChat: https://github.com/sigoden/aichat

brew install aichat

Kalosm - Rust LLM framework - https://floneum.com/kalosm/docs/
Headjack - Rust Matrix wrapped - https://docs.rs/headjack/0.4.0/headjack/index.html

# Docker

To Build: ```docker build --progress=plain -t text-rpg .```

Add ```--no-cache``` if you need to test modifying the docker image.

```docker images``` to list the available images.

To run: ```docker run -p 8080:3030 --rm --name text-rpg1 text-rpg```

For more info refer to: https://dev.to/rogertorres/first-steps-with-docker-rust-30oi

# Run release build locally

cargo build --release

cargo run --release