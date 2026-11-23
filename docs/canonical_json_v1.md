# Canonical JSON v1

v1.0 accepts one prepared comment source per project. Timestamps must resolve to
integer milliseconds relative to video start.

```json
{
  "source": {
    "sourceId": "main",
    "displayName": "Main Chat",
    "platform": "youtube"
  },
  "comments": [
    {
      "id": "c1",
      "timestampMs": 1000,
      "author": { "displayName": "Livie" },
      "body": { "text": "hello" }
    }
  ]
}
```

Rules:

- `source.sourceId` defaults to `main`.
- `comments[].id` is optional; missing IDs are generated deterministically from
  importer version, source ID, row number, timestamp, author, and body.
- `timestampMs` is preferred. `timestamp` may be milliseconds, seconds, `MM:SS`,
  or `HH:MM:SS`.
- `body` may be a string or an object with `text`.
- Duplicate IDs, invalid timestamps, missing timestamps, and empty bodies produce
  row-level diagnostics.
