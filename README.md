# Vanguard Framework Demo

A minimal, ultra-fast, and comprehensive fullstack Rust framework using `hyper` 1.0, `maud` for SSR, `tokio` for concurrency, and a custom vanilla JS router for SPA-like navigation.

## Features

- **Server-Side Rendering (SSR):** Fast HTML generation with Maud.
- **Client-Side Routing:** Intercepts links for instant page transitions without full reloads (SPA feel).
- **State Hydration:** Seamlessly pass state from Rust (server) to JavaScript (client).
- **Zero-Build Frontend:** No Webpack, no Bundler, just standard web technologies.
- **Robust Architecture:** Modular `vanguard_core` crate and separate `app`.

## Prerequisites

- Rust (latest stable)

## Running the App

1. Navigate to the project root.
2. Run the app:

```bash
cargo run -p app
```

3. Open http://localhost:3000
