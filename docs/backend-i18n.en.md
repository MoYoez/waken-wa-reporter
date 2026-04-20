<div align="right">
  <span>[<a href="./backend-i18n.en.md">English</a>]</span>
  <span>[<a href="./backend-i18n.md">简体中文</a>]</span>
</div>

# Backend i18n Scheme

The backend does not need to render full multilingual copy. It should return stable business semantics, while the frontend uses `vue-i18n` for the final user-facing text.

## Recommended Response Shape

Successful responses can keep their current structure. Failed responses should be normalized like this:

```json
{
  "success": false,
  "error": {
    "code": "device.pending_approval",
    "message": "Device is pending approval",
    "params": {
      "approvalUrl": "https://example.com/devices/123/review"
    }
  }
}
```

Field conventions:

- `code`: a stable error code used by both frontend and backend. Do not use localized copy as a protocol field.
- `message`: a backend fallback message. The frontend can display it directly when `code` has no matching translation.
- `params`: interpolation parameters for frontend translation templates.

## Frontend Mapping Rule

The frontend should resolve copy from `code` first:

```ts
const message = t(`backend.${error.code}`, error.params) || error.message;
```

Keep two fallback layers:

1. If `backend.${error.code}` exists, use the frontend locale pack.
2. If the key does not exist, fall back to the backend `message`.
3. If `message` is also missing, fall back to a generic `errors.unknown`.

## Error Code Naming

Prefer lowercase dotted names:

- `auth.token_invalid`
- `auth.token_expired`
- `device.pending_approval`
- `device.disabled`
- `activity.validation_failed`
- `inspiration.image_too_large`

Do not use raw HTTP status codes as business error codes. HTTP describes transport semantics; `code` describes business semantics.

## Server-side Guideline

Convert internal backend errors into a unified edge payload when possible:

```rust
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiErrorPayload {
    code: String,
    message: String,
    params: Option<serde_json::Value>,
}
```

Suggested status mapping:

- Validation failure: HTTP `400`
- Authentication / authorization failure: HTTP `401` / `403`
- Resource not found: HTTP `404`
- Business conflict: HTTP `409`
- Recoverable but blocked states like pending approval: HTTP `202` with `device.pending_approval`
- Unknown exception: HTTP `500`

## Why This Scheme

- The backend does not need to maintain a full multilingual copy set.
- The frontend can rely on `vue-i18n`, so language switches take effect immediately.
- Once the error codes are stable, desktop, web, and mobile clients can all reuse the same contract.
- Adding a new language only requires frontend locale updates, not backend business changes.
