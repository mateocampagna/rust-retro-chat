# Rust Retro Chat 🦀 💬

A secure, real-time messaging app built with Rust and WebSockets, featuring JWT authentication, bcrypt password hashing, and a custom retro terminal UI.

---

## 📸 Preview

### The Login Terminal
*Minimalist entry point with focus-centered design.*
![Login Screen](./assets/login-preview.png)

### The Global Room
*Real-time communication with distinct visual feedback for user messages.*
![Chat Screen](./assets/chat-preview.png)

---

## 🚀 Key Features

- **Real-Time Communication:** Instant messaging powered by WebSockets.
- **Secure Authentication:** JWT-based auth with bcrypt password hashing. Tokens are verified server-side on every WebSocket connection.
- **XSS Protection:** All HTML rendered from Markdown is sanitized via `DOMPurify` before touching the DOM.
- **Rate Limiting:** Brute-force protection on `/login` and `/register` via `tower_governor` — 5 attempts allowed, then 1 per 30 seconds per IP.
- **Persistent History:** SQLite integration via `SQLx` (async). Last 100 messages are loaded automatically on connect.
- **Rich Content Rendering:**
  - **Markdown:** Full GFM support via `Marked.js`.
  - **Syntax Highlighting:** Multi-language code blocks via `Highlight.js`.
  - **Math:** LaTeX equation rendering via `KaTeX`.
- **User Color Identity:** Deterministic per-user colors assigned server-side from the username hash — consistent across sessions and clients.
- **Structured Logging:** `tracing` + `tracing-subscriber` with `RUST_LOG` env filter support.
- **Retro UI/UX:** CRT scanlines, monochromatic palette, and pixel-perfect terminal design.

---

## 🛠️ Tech Stack

### Backend

| Component | Technology |
|---|---|
| Language | Rust 🦀 |
| Framework | Axum 0.8 |
| Async Runtime | Tokio |
| Database | SQLite + SQLx (async) |
| Auth | JWT (`jsonwebtoken`) + bcrypt |
| Rate Limiting | `tower_governor` |
| Serialization | Serde + Serde JSON |
| Logging | `tracing` + `tracing-subscriber` |

### Frontend

| Component | Technology |
|---|---|
| Logic | Vanilla JavaScript + WebSocket API |
| Sanitization | DOMPurify |
| Markdown/Math | Marked.js, Highlight.js, KaTeX |
| Styling | Pure CSS3 (variables, flexbox, custom scrollbars) |

---

## 📦 Project Structure

```
src/
├── main.rs      # Entry point: server config, DB init, routing, middleware.
├── state.rs     # Shared AppState (broadcast channel, DB pool, JWT secret).
└── handlers.rs  # Business logic: auth endpoints, WebSocket loop, static files.
```

---

## 🧠 Architecture Highlights

1. **JWT Authentication Flow:** On login, the server issues a signed JWT. The client attaches it as a query parameter when opening the WebSocket. The server validates the signature before upgrading the connection — unauthenticated requests are rejected with `401`.

2. **Server-Side Identity:** The server ignores any `name` or `color` fields sent by the client. Both are derived server-side from the verified JWT claims, preventing impersonation and XSS via crafted field values.

3. **Broadcast Architecture:** `tokio::sync::broadcast` handles one-to-many message distribution. Lagged receivers are detected and logged rather than silently dropped.

4. **Client-Side Rendering:** The server sends raw text; all Markdown, LaTeX, and syntax highlighting is processed in the browser. DOMPurify sanitizes the resulting HTML before injection.

---

## ⚡ Getting Started

### Prerequisites
- [Rust](https://rustup.rs/) (edition 2021+)

### Setup

```bash
git clone https://github.com/mateocampagna/rust-retro-chat
cd rust-retro-chat

# Create your environment file
cp .env.example .env
# Edit .env and set a strong JWT_SECRET

cargo run
```

The server starts at `http://localhost:3000`.

### Environment Variables

| Variable | Description |
|---|---|
| `JWT_SECRET` | Secret key used to sign and verify JWT tokens. Use a long random string. |

### Logging

```bash
RUST_LOG=info cargo run    # default — server events only
RUST_LOG=warn cargo run    # warnings and errors only
RUST_LOG=debug cargo run   # full verbose output
```

---

## 🗺️ Roadmap

- [x] Basic WebSocket implementation
- [x] Custom retro styling
- [x] Message history persistence (SQLite)
- [x] Markdown, code & LaTeX support
- [x] Modular codebase
- [x] User authentication (bcrypt + JWT)
- [x] Server-side user color identity
- [x] XSS protection (DOMPurify)
- [x] Rate limiting on auth endpoints
- [x] Structured logging (tracing)
- [ ] Logout button
- [ ] Online users list

---

> ⚠️ **Disclaimer:** This project was developed for learning purposes. Its goal is to explore the Rust ecosystem, WebSockets, and real-time application architecture.

---

*Created by Mateo Campagna*