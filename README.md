# Rust REST API

Production-ready REST API built with **Actix-web**, **SQLx (PostgreSQL)**, **JWT auth**, and **structured tracing**.

## Stack

| Layer | Crate |
|-------|-------|
| Web framework | `actix-web 4` |
| Database ORM | `sqlx 0.7` (PostgreSQL) |
| Authentication | `jsonwebtoken 9` |
| Password hashing | `bcrypt` |
| Logging | `tracing` + `tracing-subscriber` |
| Serialization | `serde` + `serde_json` |

## Project Structure

```
src/
├── main.rs              # Server bootstrap, routing
├── handlers/
│   ├── auth.rs          # POST /auth/register, /auth/login
│   ├── users.rs         # CRUD /users
│   └── health.rs        # GET /health
├── middleware/
│   └── auth.rs          # JWT validation middleware
├── models/
│   └── mod.rs           # User, Claims, request/response types
├── db/
│   └── mod.rs           # SQLx query functions
└── utils/
    ├── jwt.rs            # Token generation & verification
    └── logging.rs        # Tracing initializer
migrations/
└── 20240101000000_create_users.sql
```

## Quick Start

### Option A — Docker Compose (recommended)

```bash
docker compose up --build
```

The API will be available at `http://localhost:8080`.

### Option B — Local Development

**Prerequisites:** Rust 1.76+, PostgreSQL 14+, `sqlx-cli`

```bash
# 1. Install sqlx CLI
cargo install sqlx-cli --no-default-features --features mysql

# 2. Copy env file
cp .env.example .env
# Edit DATABASE_URL and JWT_SECRET
# Example: DATABASE_URL=mysql://root:password@localhost:3306/rust_api

# 3. Create database & run migrations
sqlx database create
sqlx migrate run

# 4. Run
cargo run
```

## API Endpoints

### Public

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/v1/auth/register` | Register a new user |
| `POST` | `/api/v1/auth/login` | Login, get JWT |
| `GET` | `/api/v1/health` | Health check |

### Protected (requires `Authorization: Bearer <token>`)

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/users` | List users (`?limit=20&offset=0`) |
| `GET` | `/api/v1/users/me` | Current user profile |
| `GET` | `/api/v1/users/:id` | Get user by ID |
| `PUT` | `/api/v1/users/:id` | Update user |
| `DELETE` | `/api/v1/users/:id` | Delete user |

## Example Requests

```bash
# Register
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","username":"alice","password":"secret123"}'

# Login
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"secret123"}'

# Get profile (use token from login response)
curl http://localhost:8080/api/v1/users/me \
  -H "Authorization: Bearer <token>"
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `HOST` | `0.0.0.0` | Bind address |
| `PORT` | `8080` | Port |
| `DATABASE_URL` | — | Postgres connection string |
| `JWT_SECRET` | — | Secret for signing tokens |
| `JWT_EXPIRY_HOURS` | `24` | Token lifetime |
| `RUST_LOG` | `info` | Log filter |
| `LOG_FORMAT` | `pretty` | `pretty` or `json` |
