# Verden - Social for 3D artists

This software is part of a project for the Web Programming course at UNICT.

---

Configuration to set up before starting:

1. Create a PostgreSQL database.
2. Generate a good secret string for JWT.

Variables with values as example:

```
JWT_SECRET=foobar
DATABASE_URL=postgres://user:password@localhost:5432/verden
PAGE_LIMIT=20
SAVE_FILE_BASE_PATH="./uploads"
UPLOADS_ENDPOINT="/uploads"
RUST_LOG=verden=debug,tower_http=debug
ALLOWED_HOST=localhost:3000
```
