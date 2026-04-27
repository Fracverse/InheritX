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
- `plans` - Inheritance plans with beneficiary and payout options
- `claims` - Record of plan claims by beneficiaries
- `admins`, `kyc_status`, `two_fa`, `notifications`, `logs` - Supporting tables

 Plans and beneficiary / currency

Plans store optional beneficiary bank details and payout currency preference:

- **beneficiary_name** – Full name of the beneficiary
- **bank_account_number** – Account number for fiat transfers
- **bank_name** – Name of the beneficiary's bank
- **currency_preference** – `USDC` (crypto) or `FIAT` (bank transfer)

**Currency handling:**

- **USDC**: Bank fields are optional; payout is processed as USDC transfer.
- **FIAT**: `beneficiary_name`, `bank_name`, and `bank_account_number` are required when creating a plan or when claiming with FIAT preference. Missing or invalid bank info returns a 400 error.

## Loan Simulation API

The Loan Simulation API allows borrowers to preview loan terms before committing to a loan. It calculates:

- **Required Collateral**: The minimum collateral value needed based on the collateral type's Loan-to-Value (LTV) ratio
- **Estimated Interest**: Interest calculated based on the loan amount, duration, and collateral type's annual interest rate
- **Liquidation Price**: The price at which the collateral would be liquidated if its value drops below the liquidation threshold

### Supported Collateral Types

| Collateral | LTV Ratio | Annual Interest Rate | Liquidation Threshold |
|------------|-----------|---------------------|----------------------|
| USDC       | 90%       | 5%                  | 95%                  |
| ETH        | 75%       | 8%                  | 85%                  |
| BTC        | 75%       | 8%                  | 85%                  |
| STELLAR_XLM| 60%       | 12%                 | 80%                  |

### Calculation Formulas

- **Required Collateral (USD)**: `loan_amount / LTV_ratio`
- **Collateral Quantity**: `required_collateral_usd / collateral_price_usd`
- **Estimated Interest**: `loan_amount * annual_interest_rate * (duration_days / 365)`
- **Total Repayment**: `loan_amount + estimated_interest`
- **Liquidation Price**: `(loan_amount / liquidation_threshold) / collateral_quantity`

### Endpoints

#### POST /api/loans/simulate

Create a loan simulation and store it in the database.

**Authentication**: Required (User JWT)

**Request Body**:
```json
{
  "loan_amount": 10000,
  "loan_duration_days": 30,
  "collateral_type": "ETH",
  "collateral_price_usd": 2000
}
```

**Response**:
```json
{
  "status": "success",
  "data": {
    "loan_amount": 10000,
    "loan_duration_days": 30,
    "collateral_type": "ETH",
    "collateral_price_usd": 2000,
    "required_collateral_usd": 13333.33,
    "collateral_quantity": 6.67,
    "estimated_interest": 197.26,
    "total_repayment": 10197.26,
    "liquidation_price": 1764.71,
    "loan_to_value_ratio": 0.75,
    "annual_interest_rate": 0.08,
    "liquidation_threshold": 0.85
  }
}
```

#### GET /api/loans/simulations

Get all loan simulations for the current user (limited to 50 most recent).

**Authentication**: Required (User JWT)

**Response**:
```json
{
  "status": "success",
  "data": [...],
  "count": 5
}
```

#### GET /api/loans/simulations/:simulation_id

Get a specific loan simulation by ID.

**Authentication**: Required (User JWT)

**Response**:
```json
{
  "status": "success",
  "data": {
    "id": "uuid",
    "user_id": "uuid",
    "loan_amount": 10000,
    ...
  }
}
```

### Error Responses

- **400 Bad Request**: Invalid input parameters (e.g., zero loan amount, invalid collateral type)
- **404 Not Found**: Simulation not found (for GET by ID)
- **500 Internal Server Error**: Database or server error

Plans API

- **POST /api/plans** – Create a plan (body: title, description, fee, net_amount, beneficiary_name, bank_name, bank_account_number, currency_preference). Requires FIAT bank details when currency_preference is FIAT.
- **GET /api/plans/:plan_id** – Get plan details including beneficiary info (owner only).
- **POST /api/plans/:plan_id/claim** – Record a claim (body: beneficiary_email). Payout method is determined by the plan’s currency_preference; FIAT claims require valid bank details on the plan.

## Contributing

1. Follow Rust best practices and idioms
2. Write tests for new functionality
3. Update documentation for API changes
4. Ensure code passes `cargo clippy` and `cargo fmt`

## License

This project is part of the INHERITX ecosystem.

### Admin Metrics API

- **GET /api/admin/metrics/plans** – Get comprehensive plan statistics (admin only)
  - Returns: total_plans, active_plans, expired_plans, triggered_plans, claimed_plans, and breakdown by status
