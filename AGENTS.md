# Laterfeed

Laterfeed is an application that serves as a "read-it-later" and "watch-it-later" saver and serves them as an Atom feed to be used in RSS clients.

Tech Stack: Rust, Axum, SQLx (SQLite), Utoipa (OpenAPI)

Chrome Extension lives under `extension/chrome`.

## Important Notes

- When adding dependencies, use `cargo add` to get the latest versions. Do not update `Cargo.toml` manually.
- After making changes, run `cargo clippy` and fix any issues.
- After making chages, run the tests using `cargo test`.
