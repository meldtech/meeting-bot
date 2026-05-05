# Kair Meeting Bot

HTTP service that receives **join** instructions from the Kair Voice application when someone pastes a conference link on a session. This repository holds the **meeting automation and recording** surface area so it can evolve on its own cadence.

Today’s binary is a **stub**: it validates payloads, logs intent, and returns `202 Accepted`. Wire in browser automation, headless clients, or vendor SDKs where your deployment allows.

While originally implemented for [Kair](https://kair.is/), this service is intentionally generic: any product that can send the join contract and receive audio callbacks can integrate it.

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

This service is designed to run as an independently deployed dependency of Kair Voice.

In Kair Voice `config.yml`, set **`app.meeting_bot_url`** to this service's origin **without a trailing slash** (Kair appends `/google/join`, `/zoom/join`, `/microsoft/join`, or `/jitsi/join`).

Expected flow:

1. Kair Voice sends a join request to this service.
2. This service accepts and performs meeting-platform automation/recording work.
3. This service uploads captured audio back to Kair Voice using the callback contract in [INTEGRATION.md](INTEGRATION.md).

## Roadmap / vision

Contributions welcome in these areas:

- **Multi-channel audio**: capture **separate streams** per participant (or per logical track), time-aligned, with **stable speaker or participant labels** so downstream transcription does not depend on a single mixed recording.
- **Rich metadata**: meeting identifiers, roster and join/leave events, timings, and optional platform-specific fields, exposed in a consistent shape for ingestion and product UX.
- **Jitsi**: first-class support alongside Google Meet, Zoom, and Microsoft Teams (this stub already exposes `POST /jitsi/join`; production automation still to be implemented).

Delivering multiple labelled streams or structured timelines will likely require **coordinated changes** in Kair Voice’s ingest and transcription models (beyond a single `upload-audio` file). Treat that as a later phase once single-track parity is solid.

## Contributing

Contributions are welcome from anyone building meeting automation, capture, or integration tooling.

Useful contribution types:

- New platform adapters and hardening for existing adapters.
- Reliability work (timeouts, retries, idempotency, backoff, and clearer error mapping).
- Audio pipeline improvements (track separation, labels, metadata, and upload resilience).
- Documentation, test fixtures, and reproducible local/dev workflows.

Please open an issue or pull request with context on the platform, expected behavior, and verification steps.

## Implementer checklist

If you want to implement or productionize a meeting bot on top of this service, these details usually matter most:

- Authentication model between caller and bot service (API keys/JWT/mTLS).
- Idempotency strategy for repeated join requests.
- Platform-specific capability matrix (supported meeting types, lobby flows, host permissions, recording constraints).
- Retry/backoff policy for both join attempts and audio callback uploads.
- Observability expectations (structured logs, trace IDs, metrics, and audit events).
- Failure semantics (when to return `202` vs `4xx/5xx`, and how to surface async failures).

## Sending recordings to Kair

If you are integrating this bot with [Kair](https://kair.is/), share a clear contract with your integration partner that covers endpoints, auth, payload shape, and delivery semantics.

Recommended public contract (safe to document):

- **Join endpoint**: the URL your service should call to request bot join for a meeting URL/platform.
- **Upload endpoint**: the callback URL where the final recording should be uploaded.
- **Auth**: use a short-lived bearer token for upload, scoped to one session/meeting where possible.
- **Headers**: document required headers (for example `Authorization: Bearer <token>` and content type).
- **Payload**: document required fields (platform, meeting link, external/session identifiers, optional metadata).
- **Media constraints**: document accepted codecs/formats, max file size, and whether chunked uploads are supported.
- **Retries/idempotency**: document how to retry safely and which idempotency key/identifier to include.
- **Success/failure semantics**: document expected response codes and how async failures are reported.

Suggested implementation notes:

1. Issue temporary upload tokens with limited lifetime and narrow scope.
2. Bind upload authorization to a specific session/meeting identifier.
3. Expire tokens promptly after successful upload or timeout.
4. Log request IDs for traceability across join and upload paths.
5. Keep private/internal service details out of public docs; only expose the integration contract.

## Repository status

This project is now maintained as its own standalone repository: `meldtech/meeting-bot`.

If you are extracting this service from another monorepo in the future, you can either:

- Copy the folder into a new location and run `git init`, or
- Preserve history with `git subtree split --prefix=meeting-bot -b meeting-bot-main` and push that branch to a new remote.

