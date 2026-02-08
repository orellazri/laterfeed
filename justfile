set dotenv-load := true

default:
    @just --list

# Run the project
run:
    cargo run

# Create a migration
migrate-create name:
    sqlx migrate add {{ name }}

# Run migrations
migrate-up:
    sqlx database create
    sqlx migrate run
    cargo sqlx prepare

# Release a new version:
release:
    bash scripts/release.sh
