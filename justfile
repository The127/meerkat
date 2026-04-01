set dotenv-load

dev-up:
    docker compose up -d

dev-down:
    docker compose down

dev-reset:
    docker compose down -v

dev-run: dev-up
    cargo run -p meerkat-server -- api

dev-migrate:
    cargo run -p meerkat-server -- migrate

ui-install:
    cd meerkat-ui && pnpm install

ui:
    cd meerkat-ui && pnpm dev
