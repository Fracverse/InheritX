# InheritX 

**Yield-Bearing, Fiat-Native Digital Inheritance Infrastructure on Stellar**

InheritX is a programmable, cross-border digital inheritance protocol built on the **Stellar network** using **Soroban smart contracts**. It allows individuals to secure digital assets and automatically distribute them to multiple heirs when predefined inactivity conditions are met, with automatic settlement directly to **local fiat bank accounts or mobile money** via Stellar anchors.

---

## 🌍 Core Features

### 1. Yield-Bearing Inheritance Plans
Assets locked within the inheritance vault do not sit idle. When creating a plan, owners can opt to supply their capital into yield-generating Soroban lending or liquidity vaults. Yield accumulates continuously, increasing the eventual inheritance principal distributed to heirs.

### 2. Mass Beneficiaries Payouts
InheritX facilitates distributions to multiple heirs in a single transaction. Owners set custom allocation splits (defined in basis points, e.g., 5000 bps for 50%) for any number of beneficiaries. The smart contract automatically handles the division of principal and accrued yield upon payout.

### 3. Fiat Settlement via Stellar Anchors
To bridge the gap between Web3 assets and real-world utility, beneficiaries do not need crypto wallets or blockchain literacy. Heirs can configure their payout to be off-ramped into local fiat currencies (e.g. NGN, KES, BRL, PHP, EUR, USD) and deposited directly into their local bank accounts or mobile money wallets using Stellar Anchors.

---

## 🏗 Technical Architecture

- **`contracts/inheritance-contract`**: The Soroban smart contract managing vault state, pings (proof-of-life), yield accounting, and payouts distribution.
- **`backend`**: An Axum-based Rust service simulating the Stellar Anchor off-ramp and orchestrating the database-free planning state.
- **`frontend`**: A Next.js landing page DApp featuring interactive plan configuration and a visual claim/settlement simulator.

---

## ⚡ Quick Install (Minimal)

The repo has three independent layers. Install only what you need:

```bash
# Frontend — dev server only (no e2e tests, no PWA pipeline, no image optimiser)
cd frontend && pnpm install    # .npmrc defaults optional=false (~50–80 MB lighter)
pnpm run dev

# Backend — no Redis, Prometheus, or PDF generation
cd backend && cargo build --no-default-features
cargo run --no-default-features

# Contracts only
cd contracts && cargo build --target wasm32-unknown-unknown --release
```

> **Tip:** Run `pnpm install --optional` in `frontend/` to restore optional deps:
> `@playwright/test` (e2e), `@ducanh2912/next-pwa` (PWA), `sharp` (image optimiser), `@allbridge/bridge-core-sdk` (cross-chain bridge).

---

## 🚀 Getting Started (Full)

### 1. Smart Contracts
```bash
cd contracts
cargo build --target wasm32-unknown-unknown --release
cargo test
```

### 2. Backend

Full build (Redis cache + Prometheus metrics + PDF reports):
```bash
cd backend
cargo run
```

Minimal build (no optional deps — faster compile, no Redis/Prometheus/PDF required):
```bash
cd backend
cargo run --no-default-features
```

Selective features:
```bash
cargo run -F redis-cache          # add Redis cache only
cargo run -F redis-cache,metrics  # add Redis + Prometheus
```

Optional cache tuning:
```bash
export REDIS_URL=redis://127.0.0.1:6379
export PLAN_CACHE_TTL_SECS=15
```

When Redis is enabled, `GET /api/plans` returns cache-timing headers:
`x-plan-cache-status`, `x-plan-cache-lookup-ms`, `x-plan-db-query-ms`, `x-plan-total-latency-ms`

#### API documentation (OpenAPI 3.0 + Swagger UI)

The backend exposes an auto-generated OpenAPI 3.0 document and an interactive
Swagger UI. Once the server is running:

- Swagger UI: **http://localhost:8080/swagger-ui**
- Raw OpenAPI JSON spec: **http://localhost:8080/api-docs/openapi.json**

The spec is generated at compile time from `utoipa` annotations on the Axum
handlers (`#[utoipa::path]`) and the request/response DTOs (`#[derive(ToSchema)]`,
`#[derive(IntoParams)]`). It documents path/query/header parameters, JSON request
bodies, response schemas and the authentication schemes used by the API:

- `bearer_auth` — admin JWT via `Authorization: Bearer <token>`
- `public_key_auth` + `signature_auth` — ed25519 request signature via the
  `X-Public-Key` and `X-Signature` headers

Swagger UI is purely additive: existing routes, middleware and behaviour are
unchanged.

### 3. Frontend
```bash
cd frontend
pnpm install           # lightweight install (optional deps skipped by default)
pnpm run dev           # http://localhost:3000

# For e2e tests:
pnpm install --optional
pnpm run test:e2e
```

---

## 🔐 Contribution Guidelines

InheritX is transitioning from development to testnet. The codebase is structured with placeholders, stubs, and interfaces. Contributors are invited to review open GitHub issues and implement the contract math, API handlers, anchor hooks, and frontend simulator widget.
