# stars-pay — `frontend` branch

This branch holds the **SvelteKit Mini App + overlay** sources for the
[stars-pay](../../tree/main) project. The Rust backend lives on
[`main`](../../tree/main); it embeds the prebuilt HTML produced here via
`include_str!`.

## What's here

```
tma-client/
├── src/
│   ├── mini_app/            # streamer's bot Mini App + main control bot
│   ├── layer/               # OBS donation overlay
│   ├── goal/                # goal-progress widget
│   ├── pages/               # entry HTML + bootstrap TS per build target
│   └── routes/              # SvelteKit dev routes
├── static/
├── package.json
└── svelte.config.js

build.sh                     # builds all 5 targets into tma-client/dist/
```

## Build

Prereq: [`bun`](https://bun.com/).

```bash
cd tma-client
bun install
cd ..
./build.sh
```

The script runs `bun run build` five times with different `TARGET=…`
env vars and writes the per-target HTML to
`tma-client/dist/src/pages/`.

## Publish to `main`

```bash
./build.sh
cp tma-client/dist/src/pages/*.html /tmp/
git checkout main
cp /tmp/*.html server/static/
git add server/static/*.html
git commit -m "chore: rebuild frontend assets"
git push
```

That refreshes the HTML that the Rust backend embeds at compile time.

## Build targets

| Target                | What it is                                 | Used by                                |
| --------------------- | ------------------------------------------ | -------------------------------------- |
| `mini_app`            | Donation Mini App inside the streamer bot  | Viewers tapping a donation button      |
| `main_bot_mini_app`   | Control Mini App of `@StarDonationServiceBot` | Streamers configuring their bots    |
| `layer`               | OBS browser-source overlay                 | Donation alerts on the live stream     |
| `goal_app`            | Goal-progress widget                       | Streamers running a donation goal      |
| `blocked_app`         | Read-only placeholder                      | Bots blocked for unpaid debt           |
