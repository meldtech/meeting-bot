# Kair Meeting Bot — integration with Kair Voice

This service receives **join** requests from the Kair Voice API after a user submits a meeting link on a session. Your implementation is responsible for joining the conference, capturing audio, and posting it back to Kair Voice using the issued bearer token.

## Kair Voice → meeting bot (join)

Kair Voice concatenates `app.meeting_bot_url` from its `config.yml` with a platform-specific path. **Do not** put a trailing slash on `meeting_bot_url`.

| Platform        | HTTP method and path   | `platform` value in UI/API (`POST .../meeting-bot`) |
|----------------|------------------------|-----------------------------------------------------|
| Google Meet    | `POST /google/join`    | `google_meet`                                       |
| Zoom           | `POST /zoom/join`      | `zoom`                                              |
| Microsoft Teams| `POST /microsoft/join` | `microsoft_teams`                                   |
| Jitsi          | `POST /jitsi/join`     | `jitsi`                                             |

### Request body (JSON, camelCase)

Matches Kair’s `MeetingBotRequestBody` serialisation:

| Field           | Type   | Description |
|-----------------|--------|-------------|
| `bearerToken`   | string | JWT minted by Kair for the **human user** who triggered the join. Use as `Authorization: Bearer …` when calling Kair. |
| `url`           | string | Meeting join URL. |
| `name`          | string | Display name for the bot participant (Kair currently sends a fixed value). |
| `teamId`        | string | Opaque team identifier from Kair. |
| `timezone`      | string | IANA-style label from Kair (e.g. `UTC`). |
| `userId`        | string | Opaque user handle from Kair. |
| `botId`         | string | **Session UUID** in Kair — use as `{session_id}` in callback paths. |

Example:

```json
{
  "bearerToken": "eyJhbGciOiJIUzI1NiIs...",
  "url": "https://meet.google.com/xxx-yyyy-zzz",
  "name": "Kair Cognito",
  "teamId": "KAIR",
  "timezone": "UTC",
  "userId": "kair_cognito",
  "botId": "550e8400-e29b-41d4-a716-446655440000"
}
```

### Success response

This stub returns **202 Accepted** with a small JSON body:

```json
{ "status": "accepted", "platform": "google_meet", "botId": "550e8400-e29b-41d4-a716-446655440000" }
```

Kair Voice itself returns **202** to the browser as soon as the outbound request is accepted; align your production semantics with operational needs (e.g. queue depth, idempotency).

### Errors

Invalid payloads return **400** with `{ "error": "…" }`.

## Meeting bot → Kair Voice (audio upload)

After recording, upload audio to:

`POST {kair_api_base}/sessions/{botId}/upload-audio`

- **Auth**: `Authorization: Bearer {bearerToken}` (same JWT from the join payload).
- **Content-Type**: `multipart/form-data` with fields per Kair’s `UnifiedAudioUploadPayload`:
  - `audio` — file part (WAV as used by device flow; see Kair `api/docs/device-meeting-flow.md`).
  - `chunk_sequence` (optional `i32`) — omit for a single complete file upload.
  - `chunk_duration_seconds` (optional `u32`, default 30) — metadata for chunk sizing.
  - `is_final_chunk` (optional `bool`, default false) — set appropriately if chunking.

**JWT lifetime**: Kair currently mints a short-lived token (order of hours). Long meetings may require a refreshed credential — coordinate with the Kair Voice maintainers if you need an extended or renewable token.

## Local smoke test

```bash
cargo run
# elsewhere:
curl -sS -X POST http://127.0.0.1:8090/google/join \
  -H 'Content-Type: application/json' \
  -d '{"bearerToken":"x","url":"https://meet.google.com/abc-defg-hij","name":"Bot","teamId":"KAIR","timezone":"UTC","userId":"u","botId":"550e8400-e29b-41d4-a716-446655440000"}'
```

## OpenAPI

See [`openapi.yaml`](openapi.yaml) for a machine-readable summary of the join endpoints.
