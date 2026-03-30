set dotenv-load

dev-up:
    docker compose up -d

dev-down:
    docker compose down

dev-reset:
    docker compose down -v

dev-run: dev-up
    cargo run -p meerkat-server -- api
