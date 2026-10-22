/**
 * Plans API Service
 * Handles all plan-related API calls for asset owners
 */

import { apiClient, ApiResponse, PaginatedResponse } from "./client";
import type { PlanBeneficiaryRequest, PlanResponse } from "./inheritance";

export type { PlanBeneficiaryRequest, PlanResponse };

export interface Plan {
  id: string;
  user_id: string;
  owner_address?: string;
  token_address?: string;
  amount?: number;
  title: string;
  description?: string;
  fee: number;
  net_amount: number;
  status: string;
  contract_plan_id?: number;
  distribution_method?: string;
  is_active?: boolean;
  is_paused?: boolean;
  risk_override_enabled?: boolean;
  contract_created_at?: number;
  beneficiary_name?: string;
  bank_name?: string;
  bank_account_number?: string;
  currency_preference?: string;
  created_at: string;
  updated_at: string;
  // Backend/Soroban specific details
  accrued_yield?: number;
  beneficiaries?: any[];
  grace_period_seconds?: number;
  yield_rate_bps?: number;
  earn_yield?: boolean;
  last_ping?: number;
}

/**
 * Request DTOs below mirror the Axum backend's `Plan` and `UpdatePlanRequest`
 * structs exactly (see backend/src/api.rs) so beneficiary allocations,
 * wallet addresses, and fiat anchor info are never dropped on submission.
 */
export interface CreatePlanRequest {
  /** Owner Stellar address */
  owner: string;
  /** Token contract address on Stellar */
  token: string;
  /** Amount to deposit (in token units) */
  amount: number;
  /** Full list of beneficiaries with allocations. Must sum to 10000 bps */
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

export interface UpdatePlanRequest {
  /** Full replacement list of beneficiaries with allocations. Must sum to 10000 bps */
  beneficiaries: PlanBeneficiaryRequest[];
  /** Grace period in seconds before the plan becomes claimable */
  grace_period?: number;
  /** Whether this plan earns yield via AMM / lending pools */
  earn_yield?: boolean;
  /** Annualised yield rate in basis points (e.g. 500 = 5%) */
  yield_rate_bps?: number;
}

export interface ClaimPlanRequest {
  beneficiary_email: string;
  two_fa_code: string;
}

export interface PlanDepositAddress {
  address: string;
  memo: string;
  memo_type: "text";
}

/** Total value locked for a single asset, as returned by the backend's
 * NUMERIC(78, 0) `plans.amount` column (string-encoded for precision). */
export interface AssetLockedValue {
  token_address: string;
  total_locked: string;
  plan_count: number;
}

export interface PlanStatistics {
  total_plans: number;
  active_plans: number;
  expired_plans: number;
  triggered_plans: number;
  claimed_plans: number;
  by_status: Array<{
    status: string;
    count: number;
  }>;
  locked_value_by_asset: AssetLockedValue[];
}

export interface PlanStatisticsFilters {
  /** ISO 8601 timestamp; only include plans created on or after this date */
  startDate?: string;
  /** ISO 8601 timestamp; only include plans created on or before this date */
  endDate?: string;
  /** Filter to a single token/asset address */
  assetType?: string;
}

export interface LoanLifecycleResponse {
  status: string;
  message: string;
  data?: {
    plan_id: string;
    tx_hash?: string | null;
    on_chain?: boolean;
    freeze_status?: string;
    recall_progress?: number;
    recalled_amount?: number;
    settlement_status?: string;
    settled_amount?: number;
    remaining_loaned?: number;
  };
}

export interface TriggerInfo {
  timestamp: string | null;
  freeze_status: string;
  recall_progress: number;
  settlement_status: string;
  outstanding_loans: Array<{ pool: string; amount: string; status: string }>;
}

export class PlansAPI {
  /**
   * Create a new plan
   */
  async createPlan(request: CreatePlanRequest): Promise<PlanResponse> {
    return apiClient.post<PlanResponse>("/api/plans", request);
  }

  /**
   * Get a specific plan by ID
   */
  async getPlan(planId: string): Promise<Plan> {
    const response = await apiClient.get<ApiResponse<Plan> | Plan>(
      `/api/plans/${planId}`
    );
    if (response && typeof response === "object" && "data" in response) {
      return response.data!;
    }
    return response as Plan;
  }

  /** Get a plan's Stellar deposit address and unique memo */
  async getDepositAddress(planId: string): Promise<PlanDepositAddress> {
    const response = await apiClient.get<
      ApiResponse<PlanDepositAddress> | PlanDepositAddress
    >(`/api/plans/${planId}/deposit-address`);
    if (response && typeof response === "object" && "data" in response) {
      return response.data!;
    }
    return response as PlanDepositAddress;
  }

  /**
   * Get all plans due for claim (user view)
   */
  async getDueForClaimPlans(
    page: number = 1,
    limit: number = 20
  ): Promise<PaginatedResponse<Plan>> {
    return apiClient.get<PaginatedResponse<Plan>>(
      `/api/plans/due-for-claim?page=${page}&limit=${limit}`
    );
  }

  /**
   * Get a specific plan due for claim
   */
  async getDueForClaimPlan(planId: string): Promise<Plan> {
    const response = await apiClient.get<ApiResponse<Plan>>(
      `/api/plans/due-for-claim/${planId}`
    );
    return response.data!;
  }

  /**
   * Claim a plan
   */
  async claimPlan(planId: string, request: ClaimPlanRequest): Promise<Plan> {
    const response = await apiClient.post<ApiResponse<Plan>>(
      `/api/plans/${planId}/claim`,
      request
    );
    return response.data!;
  }

  /**
   * Get plan statistics for the admin dashboard (active/expired/triggered/
   * claimed counts and total value locked per asset). Requires an admin JWT.
   */
  async getPlanStatistics(
    filters?: PlanStatisticsFilters
  ): Promise<PlanStatistics> {
    const params = new URLSearchParams();
    if (filters?.startDate) params.set("start_date", filters.startDate);
    if (filters?.endDate) params.set("end_date", filters.endDate);
    if (filters?.assetType) params.set("asset_type", filters.assetType);

    const queryString = params.toString();
    const response = await apiClient.get<ApiResponse<PlanStatistics>>(
      `/api/analytics/plan-statistics${queryString ? `?${queryString}` : ""}`
    );
    return response.data!;
  }

  /**
   * Update an existing plan's beneficiaries, grace period, or yield settings
   */
  async updatePlan(
    planId: string,
    request: UpdatePlanRequest
  ): Promise<PlanResponse> {
    return apiClient.put<PlanResponse>(`/api/plans/${planId}`, request);
  }

  /**
   * Cancel/deactivate a plan
   */
  async cancelPlan(planId: string): Promise<Plan> {
    const response = await apiClient.post<ApiResponse<Plan>>(
      `/api/plans/${planId}/cancel`
    );
    return response.data!;
  }

  /**
   * Trigger inheritance execution
   */
  async triggerPlan(planId: string): Promise<any> {
    return apiClient.post(`/api/plans/${planId}/trigger`);
  }

  /**
   * Freeze outstanding loans
   */
  async freezeLoans(planId: string): Promise<LoanLifecycleResponse> {
    return apiClient.post<LoanLifecycleResponse>(
      `/api/plans/${planId}/freeze-loans`,
      {}
    );
  }

  /**
   * Recall loans from lending pool
   */
  async recallLoans(
    planId: string,
    recallAmount?: number
  ): Promise<LoanLifecycleResponse> {
    return apiClient.post<LoanLifecycleResponse>(
      `/api/plans/${planId}/recall-loans`,
      recallAmount !== undefined ? { recall_amount: recallAmount } : {}
    );
  }

  /**
   * Liquidate collateral if loans can't be recalled
   */
  async liquidateAndSettle(planId: string): Promise<LoanLifecycleResponse> {
    return apiClient.post<LoanLifecycleResponse>(
      `/api/plans/${planId}/liquidate-settle`,
      {}
    );
  }

  /**
   * Get trigger status and progress
   */
  async getTriggerInfo(planId: string): Promise<{
    status: string;
    data: TriggerInfo;
  }> {
    return apiClient.get<{ status: string; data: TriggerInfo }>(
      `/api/plans/${planId}/trigger-info`
    );
  }

  /**
   * Keep-alive ping to reset inactivity timer
   */
  async pingKeepAlive(
    planId: string,
    signedTransaction?: string
  ): Promise<Plan> {
    const response = await apiClient.post<ApiResponse<Plan>>(
      `/api/plans/${planId}/keep-alive`,
      { signed_transaction: signedTransaction }
    );
    return response.data!;
  }

  /**
   * Get plan inactivity status
   */
  async getInactivityStatus(planId: string): Promise<{
    last_ping_timestamp: number;
    inactivity_period_days: number;
    days_until_claimable: number;
    is_claimable: boolean;
  }> {
    const response = await apiClient.get<
      ApiResponse<{
        last_ping_timestamp: number;
        inactivity_period_days: number;
        days_until_claimable: number;
        is_claimable: boolean;
      }>
    >(`/api/plans/${planId}/inactivity-status`);
    return response.data!;
  }

  /**
   * Get all plans filtered by owner's address
   */
  async getPlansByOwner(ownerAddress: string): Promise<Plan[]> {
    const response = await apiClient.get<ApiResponse<Plan[]> | Plan[]>(
      `/api/plans?owner=${encodeURIComponent(ownerAddress)}`
    );
    if (response && typeof response === "object" && "data" in response) {
      return response.data as Plan[];
    }
    return response as Plan[];
  }
}

export const plansAPI = new PlansAPI();
export default plansAPI;
