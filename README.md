# Web Chat

Real-time chat app written in Rust. Built mostly to get hands-on with Axum, async Rust, and how a WebSocket server holds state under load — the retro terminal look came after.

---

## Preview

### Login
![Login Screen](./assets/login-preview.png)

### Chat room
![Chat Screen](./assets/chat-preview.png)

---

## Features

- Real-time messaging over WebSockets, broadcast to every connected client.
- Login/register with JWT + bcrypt. The token is attached to the WebSocket connection on open and verified server-side before the upgrade completes — no valid token, no connection.
- Usernames and colors are never trusted from the client. Both are derived server-side from the verified JWT claims, so there's no way to spoof a name or inject arbitrary values through those fields.
- Per-user color is deterministic, hashed from the username, so it stays consistent across sessions and devices.
- Message history persists in SQLite via SQLx (async). The last 100 messages load automatically on connect.
- Markdown, code blocks, and LaTeX are rendered client-side (Marked.js, Highlight.js, KaTeX) and passed through DOMPurify before touching the DOM. The server only ever sends raw text — no HTML is generated server-side.
- Rate limiting on `/login` and `/register` via `tower_governor`: 5 attempts, then throttled to 1 per 30 seconds per IP.
- Structured logging with `tracing`, filterable through `RUST_LOG`.

---

## Stack

**Backend:** Rust, Axum 0.8, Tokio, SQLite + SQLx, JWT (`jsonwebtoken`) + bcrypt, `tower_governor`, `tracing`.

**Frontend:** vanilla JS + WebSocket API, DOMPurify, Marked.js, Highlight.js, KaTeX, plain CSS (no build step).

---

## Structure

```
src/
├── main.rs      # server setup, routing, middleware
├── state.rs     # shared state: broadcast channel, DB pool, JWT secret
└── handlers.rs  # auth endpoints, WebSocket loop, static files
```

---

## Running it

Requires Rust (2021 edition or later).

```bash
git clone https://github.com/mateocampagna/rust-retro-chat
cd rust-retro-chat

cp .env.example .env
# set JWT_SECRET to a long random string

cargo run
```

Server starts at `http://localhost:3000`.

```bash
RUST_LOG=info cargo run    # default
RUST_LOG=warn cargo run    # warnings and errors only
RUST_LOG=debug cargo run   # verbose
```

---

## Notes

Broadcasting uses `tokio::sync::broadcast` for one-to-many delivery. Lagged receivers are detected and logged rather than silently dropped.

Still missing: a logout button and an online users list.

---

*Mateo Campagna*