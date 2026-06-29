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

## 🚀 Getting Started

### 1. Smart Contracts
To build and test the Soroban contracts:
```bash
cd contracts
cargo build --target wasm32-unknown-unknown --release
cargo test
```

### 2. Backend
To start the Axum backend server (runs on port `3001` by default):
```bash
cd backend
cargo run
```

Optional cache tuning for hot plan reads:
```bash
export REDIS_URL=redis://127.0.0.1:6379
export PLAN_CACHE_TTL_SECS=15
```

When Redis is enabled, `GET /api/plans` now returns:
- `x-plan-cache-status`: `hit`, `miss`, `bypass`, or `error-fallback`
- `x-plan-cache-lookup-ms`
- `x-plan-db-query-ms`
- `x-plan-total-latency-ms`

Those headers are the easiest way to compare cache-hit latency against PostgreSQL fallback in local or staging runs.

### 3. Frontend
To run the Next.js development server:
```bash
cd frontend
npm run dev
```

---

## 🔐 Contribution Guidelines

InheritX is transitioning from development to testnet. The codebase is structured with placeholders, stubs, and interfaces. Contributors are invited to review open GitHub issues and implement the contract math, API handlers, anchor hooks, and frontend simulator widget.
