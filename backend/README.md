# INHERITX Backend

A Rust-based backend for the INHERITX crypto system, built on Stellar network with Soroban smart contracts.

## Architecture

The INHERITX backend provides the following services:

- **Identity & Wallet Service**: User management and Stellar address resolution
- **Anchor Integration Service**: SEP-24/SEP-31 integration for fiat on/off ramps
- **Compliance & Risk Engine**: Sanctions screening and transaction monitoring
- **Transaction Log & Audit Service**: Immutable audit trails
- **Admin Dashboard API**: System monitoring and management
- **Indexer / Ledger Listener**: Stellar network event monitoring

## Quick Start

### Prerequisites

- Rust 1.70+
- PostgreSQL 13+
- Stellar CLI (optional, for development)

### Setup

1. **Clone and navigate to backend:**

   ```bash
   cd backend
   ```

2. **Install dependencies:**

   ```bash
   cargo build
   ```

3. **Database setup:**

   ```bash
   # Create PostgreSQL database
   createdb inheritx

   # Set environment variables
   cp env.example .env
   # Edit .env with your database URL and other settings
   ```

4. **Run migrations:**

   ```bash
   cargo run --bin migrate
   ```

5. **Start the server:**
   ```bash
   cargo run
   ```

The server will start on `http://localhost:3000`.

## Inheritance Plan Retrieval API

The backend now supports retrieval of inheritance plans for both authenticated users and administrators.

### Auth headers

- User endpoints require either:
  - `x-user-id: <uuid>`
  - or `Authorization: Bearer <uuid>` (UUID user id)
- Admin endpoints require:
  - `x-admin-id: <uuid>`

### Endpoints

- `GET /plans/:id` - Get one plan for the authenticated user.
- `GET /plans` - Get all plans (active and inactive) for the authenticated user.
- `GET /plans/pending` - Get all pending plans for the authenticated user.
- `GET /admin/plans` - Get all plans across all users (admin only).
- `GET /admin/plans/pending` - Get all pending plans across all users (admin only).

## Development

### Running Tests

```bash
cargo test
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

### Database Migrations

Migrations are automatically run on startup. To manually run migrations:

```bash
cargo run --bin migrate
```

## Security Considerations

- JWT tokens expire after 24 hours by default
- All user funds remain non-custodial
- Transactions are signed client-side
- Compliance checks are performed on all transactions
- Audit logs are immutable and comprehensive

## Deployment

The backend is designed to be deployed as a single binary:

```bash
cargo build --release
./target/release/inheritx-backend
```

Use environment variables or config files to configure for different environments.

## Architecture Details

### Service Layer

The backend follows a modular service architecture:

- Each service handles a specific domain (identity, payments, compliance, etc.)
- Services are stateless and receive database connections via dependency injection
- All business logic is contained within service methods

### Middleware

- **Authentication**: JWT-based user authentication
- **Authorization**: Role-based access control
- **Metrics**: Prometheus metrics collection
- **Request ID**: Request tracing and correlation
- **CORS**: Cross-origin resource sharing

### Database Schema

The PostgreSQL database contains the following main tables:

- `users` - User accounts and Stellar addresses
- `merchants` - Merchant configurations and vaults
- `payments` - Payment transactions
- `transfers` - User-to-user transfers
- `withdrawals` - Withdrawal transactions
- `balances` - Account balances
- `audit_logs` - Audit trail
- `bridge_transactions` - Cross-chain bridge transactions

## Contributing

1. Follow Rust best practices and idioms
2. Write tests for new functionality
3. Update documentation for API changes
4. Ensure code passes `cargo clippy` and `cargo fmt`

## License

This project is part of the INHERITX ecosystem.
