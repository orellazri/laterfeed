set dotenv-load := true

default:
    @just --list

# Run the project
run:
    cargo run

# Run tests
test:
    cargo test

# Create a migration
migrate-create name:
    sqlx migrate add {{ name }}

# Run migrations
migrate-up:
    sqlx database create
    sqlx migrate run
    cargo sqlx prepare

# Release a new version
release:
    bash scripts/release.sh

# Release a new version of the extension (chrome or firefox)
release-extension browser:
    bash scripts/release-extension.sh {{ browser }}
