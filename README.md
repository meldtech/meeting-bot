# Kair Meeting Bot

HTTP service that receives **join** instructions from the Kair Voice application when someone pastes a conference link on a session. This repository holds the **meeting automation and recording** surface area so it can evolve on its own cadence.

Today’s binary is a **stub**: it validates payloads, logs intent, and returns `202 Accepted`. Wire in browser automation, headless clients, or vendor SDKs where your deployment allows.

- **Integration contract**: [INTEGRATION.md](INTEGRATION.md) (Kair → bot join JSON, bot → Kair `upload-audio`).
- **OpenAPI**: [openapi.yaml](openapi.yaml).

## Run locally

```bash
cp .env.example .env   # optional
cargo run
```

Health check: `GET http://127.0.0.1:8090/health`

Docker:

```bash
docker compose up --build
```

Environment variables are described in [`.env.example`](.env.example).

## Configure Kair Voice

In Kair Voice `config.yml`, set **`app.meeting_bot_url`** to this service’s origin **without a trailing slash** (Kair appends `/google/join`, `/zoom/join`, etc.). See [api/docs/meeting-bot-service.md](../api/docs/meeting-bot-service.md) in the Kair Voice tree.

## Roadmap / vision

Contributions welcome in these areas:

- **Multi-channel audio**: capture **separate streams** per participant (or per logical track), time-aligned, with **stable speaker or participant labels** so downstream transcription does not depend on a single mixed recording.
- **Rich metadata**: meeting identifiers, roster and join/leave events, timings, and optional platform-specific fields, exposed in a consistent shape for ingestion and product UX.
- **Jitsi**: first-class support alongside Google Meet, Zoom, and Microsoft Teams (this stub already exposes `POST /jitsi/join`; production automation still to be implemented).

Delivering multiple labelled streams or structured timelines will likely require **coordinated changes** in Kair Voice’s ingest and transcription models (beyond a single `upload-audio` file). Treat that as a later phase once single-track parity is solid.

## Publishing as a separate Git repository

This directory lives inside the Kair Voice monorepo. To maintain a **dedicated remote** for collaborators, either:

- Copy or move this folder elsewhere and run `git init`, or
- From the monorepo root: `git subtree split --prefix=meeting-bot -b meeting-bot-main` and push that branch to a new remote.

