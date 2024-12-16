# Text RPG

Install AIChat: https://github.com/sigoden/aichat

brew install aichat

Kalosm - Rust LLM framework - https://floneum.com/kalosm/docs/
Headjack - Rust Matrix wrapped - https://docs.rs/headjack/0.4.0/headjack/index.html

## Install

### Config

Copy ```config.example.yml``` and rename to ```config.yml```. Set up the ```config.yml``` as required.

If you have no rooms specified the bot will respond in ALL rooms.

## Docker

## Image

To Build: ```docker build --progress=plain -t text-rpg .```

Add ```--no-cache``` if you need to test modifying the docker image.

```docker images``` to list the available images.

To run: ```docker run -p 443:443 -v /etc/ssl:/etc/ssl:ro -v .:/data --rm --name text-rpg1 text-rpg```

For more info refer to: https://dev.to/rogertorres/first-steps-with-docker-rust-30oi

### Compose

```compose.yml``` is provided to help setup the docker image and supply the appropriate files needed.

Build the docker image first.

Copy ```config.yml``` into ```~/Docker/text-rpg```.

Copy your aichat config (More info here: ```https://github.com/sigoden/aichat/wiki/Configuration-Guide```) into ```~/Docker/text-rpg``` and call it ```aichat.config.yml``.

```docker-compose -f compose.yml up --build```.

Note that once you run docker-compose, the container will start and will be started when ever docker is started on your machine.


## Local release builds

### Build and run release

```cargo build --release```

```cargo run --release```

## TODO list

* Reduce number of prompts.
* Stop players who are dead from acting.
* AI image generation.