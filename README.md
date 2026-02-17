<img src="assets/logo.png" alt="Laterfeed Logo" width="180px" align="right" />

**Laterfeed** lets you save articles and videos to read or watch later.<br>
It exposes your saved links as an <a href="<https://en.wikipedia.org/wiki/Atom_(web_standard)>)">Atom feed</a>, so you can consume them in any RSS reader.<br>

<br>

## Features

- Save articles and videos with a single API call
- Automatic metadata extraction (title and description) from saved URLs
- Atom feed generation for use with any RSS reader
- [Chrome](https://chromewebstore.google.com/detail/laterfeed/lehgeakcddcjigboiegoogbgaohcfhfn) & [Firefox](https://addons.mozilla.org/en-US/firefox/addon/laterfeed/) extensions for one-click saving from the browser
- OpenAPI documentation with interactive Scalar UI at `/docs`
- Simple token authentication
- SQLite database - no external dependencies

## Getting Started

### Run with Docker (Recommended)

Generate an authentication token using this method or any other method that you prefer:

```bash
openssl rand -hex 32
```

Run the container:

```bash
docker run -d -p 8000:8000 \
  -v $(pwd)/data:/data \
  -e PORT=8000 \
  -e DATABASE_URL=sqlite:/data/data.db \
  -e BASE_URL=http://localhost:8000 \
  -e AUTH_TOKEN=changeme \
  reaperberri/laterfeed:latest
```

Laterfeed is configured via environment variables:

| Variable         | Description                                              | Example                 |
| ---------------- | -------------------------------------------------------- | ----------------------- |
| `PORT`           | Port the server listens on                               | `8000`                  |
| `DATABASE_URL`   | SQLite connection string                                 | `sqlite:data.db`        |
| `BASE_URL`       | Public URL of the server (used in feed links)            | `http://localhost:8000` |
| `AUTH_TOKEN`     | Bearer token for authenticated endpoints                 | `changeme`              |
| `RETENTION_DAYS` | Auto-delete entries older than this many days (optional) | `30`                    |
| `MAX_ENTRIES`    | Keep only the N most recent entries (optional)           | `500`                   |

### API Routes

| Method   | Path            | Auth | Description                       |
| -------- | --------------- | ---- | --------------------------------- |
| `GET`    | `/health`       | No   | Health check                      |
| `GET`    | `/feed`         | No   | Get saved entries as an Atom feed |
| `GET`    | `/entries`      | No   | List all entries as JSON          |
| `POST`   | `/entries`      | Yes  | Add a new entry                   |
| `DELETE` | `/entries/{id}` | Yes  | Delete an entry                   |
| `GET`    | `/docs`         | No   | Interactive OpenAPI documentation |

### Retention / Cleanup

By default, saved entries are kept forever. You can configure automatic cleanup using these optional environment variables:

- **`RETENTION_DAYS`** - Entries older than this many days are automatically deleted. Set to `0` or leave unset to disable.
- **`MAX_ENTRIES`** - Only the N most recent entries are kept. Older entries beyond this limit are automatically deleted. Set to `0` or leave unset to disable.

Both options can be used together. The cleanup task runs every hour in the background. Entries can also be deleted manually via the `DELETE /entries/{id}` endpoint.

## Development

Requires [Rust](https://www.rust-lang.org/tools/install) and [just](https://github.com/casey/just).

Copy `.env.example` to `.env` and update the values where necessary.

Run:

```bash
just migrate-up
just run
```
