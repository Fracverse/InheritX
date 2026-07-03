/**
 * Inheritance Plans API Client
 *
 * Communicates with the Axum backend for inheritance plan management.
 * Routes requiring ed25519 signature authentication are automatically
 * signed via the SignatureAuth provider configured on the ApiClient.
 *
 * Backend routes:
 *   POST   /api/plans             – Create a plan (signature auth)
 *   GET    /api/plans             – Fetch plans (public, optional filters)
 *   POST   /api/plans/ping        – Ping a plan (signature auth)
 *   POST   /api/plans/payout      – Trigger payout (signature auth)
 *   GET    /api/anchor/payout-status – Fetch payout status (public)
 */

import { apiClient, RequestConfig } from "./client";

// ─── Request DTOs ────────────────────────────────────────────────────────────

export interface PlanBeneficiaryRequest {
  /** Stellar wallet address of the beneficiary */
  address: string;
  /** Human-readable name / label */
  name: string;
  /** Allocation in basis points (e.g. 5000 = 50%). Must sum to 10000 across all beneficiaries */
  allocation_bps: number;
  /** Fiat anchor / off-ramp reference string */
  fiat_anchor_info: string;
}

export interface CreatePlanRequest {
  /** Owner Stellar address */
  owner: string;
  /** Token contract address on Stellar */
  token: string;
  /** Amount to deposit (in token units) */
  amount: number;
  /** List of beneficiaries with allocations */
  beneficiaries: PlanBeneficiaryRequest[];
  /** Unix timestamp of the last liveness ping */
  last_ping: number;
  /** Grace period in seconds before the plan becomes claimable */
  grace_period: number;
  /** Whether this plan earns yield via AMM / lending pools */
  earn_yield: boolean;
  /** Annualised yield rate in basis points (e.g. 500 = 5%) */
  yield_rate_bps: number;
  /** Whether the plan is active immediately */
  is_active: boolean;
}

export interface PingRequest {
  /** Owner Stellar address */
  owner: string;
  /** Hex-encoded ed25519 signature of the message */
  signature: string;
  /** The message that was signed (timestamp or nonce) */
  message: string;
}

export interface PayoutRequest {
  /** Owner Stellar address */
  owner: string;
}

// ─── Response DTOs ───────────────────────────────────────────────────────────

export interface BeneficiaryResponse {
  /** Unique beneficiary record ID */
  id: string;
  /** Parent plan ID */
  plan_id: string;
  /** Stellar wallet address */
  wallet_address: string;
  /** Allocation in basis points */
  allocation_bps: number;
  /** Fiat anchor / off-ramp reference */
  fiat_anchor_info: string;
}

export interface PlanResponse {
  /** Unique plan ID (UUID) */
  id: string;
  /** Owner Stellar address */
  owner_address: string;
  /** Token contract address */
  token_address: string;
  /** Locked amount (string-encoded Decimal for precision) */
  amount: string;
  /** Grace period in seconds */
  grace_period: number;
  /** Grace period in seconds (duplicate for backwards compat) */
  grace_period_seconds: number;
  /** Whether yield accrual is enabled */
  earn_yield: boolean;
  /** Unix timestamp of the last ping */
  last_ping: number;
  /** Whether the plan is currently active */
  is_active: boolean;
  /** Plan status (e.g. ACTIVE, CLAIMABLE, SETTLED) */
  status: string;
  /** Yield rate in basis points */
  yield_rate_bps: number;
  /** Accrued yield amount */
  accrued_yield: number;
  /** ISO-8601 creation timestamp */
  created_at: string;
  /** Beneficiaries attached to this plan */
  beneficiaries: BeneficiaryResponse[];
}

export interface PingResponse {
  /** Owner address echoed back */
  owner: string;
  /** Current plan status */
  status: string;
  /** Principal + accrued yield balance (string-encoded Decimal) */
  virtual_balance: string;
}

export interface PayoutRow {
  /** Unique payout record ID */
  id: string;
  /** Parent plan ID */
  plan_id: string;
  /** Beneficiary wallet address */
  beneficiary_address: string;
  /** Payout amount (string-encoded Decimal) */
  amount: string;
  /** Payout type (fiat, crypto, etc.) */
  payout_type: string;
  /** Payout status (PENDING, COMPLETED, FAILED) */
  status: string;
  /** ISO-8601 creation timestamp */
  created_at: string;
}

export interface PayoutStatusResponse {
  /** Array of payout records */
  data: PayoutRow[];
  /** Current page number (1-indexed) */
  page: number;
  /** Page size */
  page_size: number;
  /** Total number of records matching the query */
  total: number;
}

// ─── Query params ────────────────────────────────────────────────────────────

export interface GetPlansQuery {
  /** Filter by owner address */
  owner?: string;
  /** Filter by beneficiary address */
  beneficiary?: string;
}

export interface GetPayoutsQuery {
  /** Filter by beneficiary address */
  beneficiary_address?: string;
  /** Page number (1-indexed, default: 1) */
  page?: number;
  /** Page size (clamped 1-100, default: 20) */
  page_size?: number;
}

// ─── API Client ──────────────────────────────────────────────────────────────

export class InheritanceAPI {
  /**
   * Create a new inheritance plan.
   *
   * Requires signature auth (X-Public-Key + X-Signature headers).
   * Call `apiClient.setSignatureAuth({ publicKey, sign })` before invoking.
   */
  async createPlan(
    request: CreatePlanRequest,
    config?: RequestConfig
  ): Promise<PlanResponse> {
    return apiClient.post<PlanResponse>("/api/plans", request, config);
  }

  /**
   * Fetch plans with optional filters.
   *
   * Public endpoint — no auth required.
   */
  async getPlans(
    query?: GetPlansQuery,
    config?: RequestConfig
  ): Promise<PlanResponse[]> {
    const params = new URLSearchParams();
    if (query?.owner) params.set("owner", query.owner);
    if (query?.beneficiary) params.set("beneficiary", query.beneficiary);

    const qs = params.toString();
    const endpoint = qs ? `/api/plans?${qs}` : "/api/plans";

    return apiClient.get<PlanResponse[]>(endpoint, config);
  }

  /**
   * Send a liveness ping to keep the plan active.
   *
   * Requires signature auth (X-Public-Key + X-Signature headers).
   * The request body is automatically signed by the configured SignatureAuth.
   */
  async pingPlan(
    request: PingRequest,
    config?: RequestConfig
  ): Promise<PingResponse> {
    return apiClient.post<PingResponse>(
      "/api/plans/ping",
      request,
      config
    );
  }

  /**
   * Trigger payout execution for a plan.
   *
   * Requires signature auth (X-Public-Key + X-Signature headers).
   */
  async triggerPayout(
    request: PayoutRequest,
    config?: RequestConfig
  ): Promise<void> {
    return apiClient.post<void>("/api/plans/payout", request, config);
  }

  /**
   * Fetch payout status records.
   *
   * Public endpoint — no auth required.
   */
  async getPayoutStatus(
    query?: GetPayoutsQuery,
    config?: RequestConfig
  ): Promise<PayoutStatusResponse> {
    const params = new URLSearchParams();
    if (query?.beneficiary_address)
      params.set("beneficiary_address", query.beneficiary_address);
    if (query?.page !== undefined) params.set("page", String(query.page));
    if (query?.page_size !== undefined)
      params.set("page_size", String(query.page_size));

    const qs = params.toString();
    const endpoint = qs
      ? `/api/anchor/payout-status?${qs}`
      : "/api/anchor/payout-status";

    return apiClient.get<PayoutStatusResponse>(endpoint, config);
  }
}

// Singleton instance
export const inheritanceAPI = new InheritanceAPI();
export default inheritanceAPI;
