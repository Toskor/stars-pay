# stars-pay

Telegram Stars donation service for streamers. Viewers send Telegram Stars
through a streamer's bot; the donation pops up on the live stream in real
time via a browser-source overlay (OBS / Streamlabs / etc.).

End-user flow:

1. A streamer opens `@StarDonationServiceBot`, creates their own bot through
   the Mini App, configures donation buttons, and copies an OBS overlay URL.
2. A viewer opens the streamer's bot, taps a donation button, pays in Stars.
3. Telegram delivers a webhook to this server, which credits the bot,
   broadcasts the donation to all overlays subscribed to that bot's room,
   and the alert animates on the live stream.

The Rust backend is the focus of this repository.

---

## Tech stack

- **Rust** (edition 2021), `tokio` async runtime
- **axum 0.7** — HTTP routing + extractors
- **fastwebsockets** — per-bot WebSocket rooms (overlay fan-out)
- **rusqlite / async-rusqlite** — single-file SQLite persistence
- **hyper 1** + custom HTTP client — outbound calls to Telegram and S3
  (no `reqwest`, kept the dependency surface small)
- **aws-sdk-s3** — Scaleway-compatible S3 for hosting Mini App / overlay
  static pages and user-uploaded images
- **tracing** + `tracing-subscriber` — structured logging, `RUST_LOG` filter
- **lru** — bot record cache in front of SQLite
- **hmac / sha2** — Telegram Mini App `initData` signature validation
- **prost** + Protobuf — binary wire format on the overlay WebSocket
  (see [`server/proto/events.proto`](server/proto/events.proto));
  protoc is vendored via `protoc-bin-vendored`, no system install needed

Frontend (SvelteKit Mini App + overlay) lives on the [`frontend`](../../tree/frontend)
branch. Its prebuilt HTML is checked into this branch under
`server/static/` and embedded into the binary via `include_str!`, so the
server runs as a single self-contained executable — no separate frontend
process to start.

---

## Architecture

```
                Telegram                      Streamer's OBS
                   │                                │
                   │ HTTPS webhook                  │ WSS
                   ▼                                ▼
        ┌─────────────────────────────────────────────────┐
        │                  axum router                    │
        │  /:bot_id/webhook   /ws/:bot     /:bot_id/...   │
        └────┬──────────┬──────────────┬───────────────┬──┘
             │          │              │               │
             ▼          ▼              ▼               ▼
        webhook      handlers      ws_server        bot/admin
        parsing    (typed errors)  (broadcast       management
             │                      rooms)          (Mini App)
             ▼
         AppState   ────►   LRU cache  ────►   SQLite (DBBot)
             │
             ├────►   S3 (mini-app HTML, overlays, avatars)
             └────►   Telegram Bot API (HTTP)
```

Donation path (webhook → overlay):

```
Telegram → POST /:bot_id/webhook
        → webhook::parse_update
        → AppState::increase_stars_debt_for  (SQLite)
        ∥
        → AppState::send_event_to_room_members
        → broadcast::Sender<RoomMessage>     (per-bot room)
        → handle_client  ─ ws frame ─►  overlay (browser source)
```

Auth model:

- Mini App requests carry `X-Telegram-InitData` and are validated against
  the bot token using HMAC-SHA256 per the [Telegram Mini App spec][1].
- Telegram webhooks carry `X-Telegram-Bot-Api-Secret-Token`, compared to a
  per-bot secret set when the webhook is registered.
- Three axum extractors (`AuthenticatedUser`, `BotAccess`,
  `BotOwnerOrAdmin`) centralise this in `handlers/auth.rs` so handlers
  stay focused on business logic.

[1]: https://core.telegram.org/bots/webapps#validating-data-received-via-the-mini-app

---

## Layout

```
server/src/
├── main.rs           # axum router, tokio entry, tracing init
├── config.rs         # env → Config struct, fails fast at boot
├── error.rs          # AppError + IntoResponse, single mapping point
├── app_state.rs      # Arc<AppState>: cache, db, rooms, s3, config
├── db.rs             # SQLite layer (async-rusqlite)
├── handlers.rs       # ws_handler, sound_handler + submodule re-exports
├── handlers/
│   ├── auth.rs       # FromRequestParts extractors for Mini App auth
│   ├── bot.rs        # bot lifecycle, config, admins, avatar proxy
│   ├── webhook.rs    # Telegram webhook entrypoint
│   └── layer.rs      # overlay token + test donation
├── ws_server.rs      # per-client loop, room broadcast (fastwebsockets)
├── proto.rs          # generated protobuf types + WSEvent → ServerMessage
├── tg_api.rs         # Telegram Bot API client (hyper-based)
├── http.rs           # outbound HTTP client + TLS + gzip
├── s3_api.rs         # Scaleway S3 client setup
├── main_bot.rs       # main control bot update handling
├── json.rs           # request/response/WS payload types
└── static/           # prebuilt frontend HTML (embedded via include_str!)

db/                   # SQLite file (auto-created)
```

Frontend sources are on the `frontend` branch (not on `main`) to keep the
default view focused on the Rust backend.

---

## Quick start

Prerequisites: Rust stable, a public HTTPS URL for Telegram to reach
(ngrok works), an S3-compatible bucket (Scaleway is what this was built
against; AWS S3 works too).

```bash
# 1. Copy the env template and fill in your secrets
cp .env.example .env
$EDITOR .env

# 2. Run the server (Mini App static files are already embedded)
cargo run --manifest-path server/Cargo.toml
```

### Rebuilding the frontend

The Svelte sources are on the `frontend` branch. To regenerate the embedded
HTML:

```bash
git checkout frontend
./build.sh                                    # bun run build × 5 targets
cp tma-client/dist/src/pages/*.html /tmp/
git checkout main
cp /tmp/*.html server/static/
```

Set `RUST_LOG=info,tg_stars=debug` (already the default if unset) to see
structured logs.

### Environment variables

See [`.env.example`](.env.example). The boot path fails loudly if a required
variable is missing.

---

## Notable design choices

- **Custom hyper-based HTTP client** instead of `reqwest`: a deliberate
  choice to keep the dependency tree small and to deal directly with
  connection reuse, gzip, and TLS for the Telegram and S3 paths.
- **Static frontend embedded via `include_str!`**: the server is a single
  binary; nothing to serve from disk in production, deploy is `scp` + run.
- **SQLite is fine.** ~100 bots, ~100 admins each. An LRU cache in front
  of the DB removes the hot-path read cost. A `bot_admins` table would be
  the natural next step if scale demanded it (today the admin list is a
  JSON column).
- **Typed errors at the HTTP boundary** (`AppError` → `IntoResponse`),
  `anyhow` inside the business layer. Handlers stay short:

  ```rust
  pub async fn remove_bot_admin(
      State(state): State<Arc<AppState>>,
      BotAccessWithPayload { access, payload }: BotAccessWithPayload<_>,
  ) -> AppResult<Json<Value>> {
      if access.role != UserRole::Owner {
          return Err(AppError::Forbidden("only owner can remove admin".into()));
      }
      state.remove_bot_admin(access.user.id, &payload.bot_id, payload.admin_id).await?;
      Ok(Json(json!({"status": "success"})))
  }
  ```

- **Per-bot WebSocket rooms** with `tokio::sync::broadcast`. The room is
  created on the first subscriber and torn down on the last unsubscribe;
  `RwLock<HashMap<_, _>>` is the right shape because reads (every donation
  event) dominate writes (subscribe/unsubscribe).

- **Binary protobuf on the overlay WebSocket.** Events go out as
  `Frame::binary` carrying an encoded
  [`ServerMessage`](server/proto/events.proto) (oneof: donation / goal /
  error). The schema is versioned via the `tg_stars.v1` package; adding a
  variant is a non-breaking change for older clients. `prost-build` is
  driven from `build.rs` and uses a vendored `protoc`, so the build
  doesn't depend on a system protobuf install.

---

## Roadmap

A few things deliberately left out, in priority order:

- Normalize `bot_admins` into its own table (current JSON column works at
  this scale but loses you `JOIN`s).
- Proper migration tool (currently the schema is created in `db.rs`).
- Replace the bot token field with a salted hash so a DB leak doesn't
  hand attackers control of every streamer's bot.
- Per-bot rate limiting on the webhook endpoint.
- Prometheus metrics + `/healthz`.

---

## License

This is a portfolio / pet project. No license granted; ask before reusing.
