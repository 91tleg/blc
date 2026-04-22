# BLC — API Contract

Base URL: `https://<api-id>.execute-api.<region>.amazonaws.com/<stage>`

All request/response bodies are `application/json`.  
Admin routes require `Authorization: Bearer <jwt>` header.

---

## Auth

### POST /auth/login
Admin only. Exchange the global password for a short-lived JWT.

**Request**
```json
{
  "password": "<ADMIN_GLOBAL_PASSWORD>"
}
```

**Response `200`**
```json
{
  "token": "<jwt>",
  "expires_at": "2025-04-21T08:00:00Z"
}
```

**Errors**
| Status | code | Reason |
|--------|------|--------|
| 401 | `INVALID_CREDENTIALS` | Wrong password |

---

## Events

### GET /events
List all events. Public — no auth required.

**Query params**

| Param | Type | Description |
|-------|------|-------------|
| `limit` | int (default 20, max 100) | Page size |
| `cursor` | string | Pagination cursor from previous response |

**Response `200`**
```json
{
  "events": [
    {
      "event_id": "evt_xyz789",
      "title": "Spring Hackathon",
      "description": "Build something cool in 24 hours.",
      "location": "Engineering Hall 101",
      "starts_at": "2025-04-15T09:00:00Z",
      "ends_at": "2025-04-16T09:00:00Z",
      "capacity": 100,
      "registered_count": 42,
      "poster_url": "https://s3.amazonaws.com/...",
      "created_at": "2025-03-01T12:00:00Z"
    }
  ],
  "next_cursor": "dXNlcjox"
}
```

---

### GET /events/{event_id}
Get a single event. Public — no auth required.

**Response `200`** — same shape as a single item in the list above.

**Errors**
| Status | code | Reason |
|--------|------|--------|
| 404 | `EVENT_NOT_FOUND` | No event with that ID |

---

### POST /events
Create a new event. **Admin only.**

**Request**
```json
{
  "title": "Spring Hackathon",
  "description": "Build something cool in 24 hours.",
  "location": "Engineering Hall 101",
  "starts_at": "2025-04-15T09:00:00Z",
  "ends_at": "2025-04-16T09:00:00Z",
  "capacity": 100,
  "poster_upload_key": "posters/evt_xyz789.jpg"
}
```

`poster_upload_key` is optional — obtain it from `GET /events/poster-upload-url` first,
upload directly to S3, then include the key here.

**Response `201`**
```json
{
  "event_id": "evt_xyz789",
  "title": "Spring Hackathon",
  "description": "Build something cool in 24 hours.",
  "location": "Engineering Hall 101",
  "starts_at": "2025-04-15T09:00:00Z",
  "ends_at": "2025-04-16T09:00:00Z",
  "capacity": 100,
  "registered_count": 0,
  "poster_url": null,
  "created_at": "2025-04-21T10:00:00Z"
}
```

**Errors**
| Status | code | Reason |
|--------|------|--------|
| 403 | `FORBIDDEN` | Missing or invalid admin token |
| 422 | `VALIDATION_ERROR` | Missing / invalid fields |

---

### GET /events/poster-upload-url
Get a pre-signed S3 PUT URL to upload a poster image. **Admin only.**

**Query params**

| Param | Type | Description |
|-------|------|-------------|
| `filename` | string | e.g. `hackathon.jpg` |
| `content_type` | string | e.g. `image/jpeg` |

**Response `200`**
```json
{
  "upload_url": "https://s3.amazonaws.com/blc-posters/...?X-Amz-Signature=...",
  "poster_upload_key": "posters/evt_xyz789.jpg",
  "expires_in": 300
}
```

**Upload flow**
1. `GET /events/poster-upload-url` → receive `upload_url` + `poster_upload_key`
2. `PUT <upload_url>` with the raw image bytes (done client-side, bypasses Lambda)
3. Include `poster_upload_key` in `POST /events`

---

## Registrations

### POST /events/{event_id}/registrations
Sign up for an event. **No auth required** — name + email only.

**Request**
```json
{
  "full_name": "Jane Doe",
  "email": "jane@school.edu",
  "phone_number": "+12065550100"
}
```

`phone_number` is optional. If provided, must contain 10–15 digits (E.164).

**Response `201`**
```json
{
  "registration_id": "reg_abc001",
  "event_id": "evt_xyz789",
  "full_name": "Jane Doe",
  "email": "jane@school.edu",
  "phone_number": "+12065550100",
  "registered_at": "2025-03-10T14:22:00Z"
}
```

**Errors**
| Status | code | Reason |
|--------|------|--------|
| 400 | `ALREADY_REGISTERED` | This email is already registered for this event |
| 400 | `EVENT_FULL` | Capacity reached |
| 404 | `EVENT_NOT_FOUND` | No event with that ID |
| 422 | `VALIDATION_ERROR` | Missing / invalid fields (includes invalid phone format) |

---

### GET /events/{event_id}/registrations
List everyone signed up for an event. **Admin only.**

**Query params**

| Param | Type | Description |
|-------|------|-------------|
| `limit` | int (default 50, max 200) | Page size |
| `cursor` | string | Pagination cursor |

**Response `200`**
```json
{
  "registrations": [
    {
      "registration_id": "reg_abc001",
      "full_name": "Jane Doe",
      "email": "jane@school.edu",
      "phone_number": "+12065550100",
      "registered_at": "2025-03-10T14:22:00Z"
    }
  ],
  "next_cursor": "dXNlcjox",
  "total": 42
}
```

---

## Error Envelope

All error responses share this shape:

```json
{
  "error": {
    "code": "EVENT_FULL",
    "message": "This event has reached its capacity."
  }
}
```

---

## JWT Claims (admin token only)

```json
{
  "sub": "admin",
  "role": "admin",
  "exp": 1722470400
}
```

Token lifetime: **8 hours**.

---

## Route Summary

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/auth/login` | — | Admin login, returns JWT |
| GET | `/events` | — | List events |
| GET | `/events/{event_id}` | — | Get single event |
| POST | `/events` | Admin | Create event |
| GET | `/events/poster-upload-url` | Admin | Get S3 pre-signed upload URL |
| POST | `/events/{event_id}/registrations` | — | Sign up for an event |
| GET | `/events/{event_id}/registrations` | Admin | View all registrants |
