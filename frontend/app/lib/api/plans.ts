/**
 * Plans API Service
 * Handles all plan-related API calls for asset owners
 */

import { apiClient, ApiResponse, PaginatedResponse } from "./client";

export interface Plan {
  id: string;
  user_id: string;
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
}

export interface CreatePlanRequest {
  title: string;
  description?: string;
  fee: number;
  net_amount: number;
  beneficiary_name?: string;
  bank_account_number?: string;
  bank_name?: string;
  currency_preference: string;
  two_fa_code: string;
}

export interface ClaimPlanRequest {
  beneficiary_email: string;
  two_fa_code: string;
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
}

export class PlansAPI {
  /**
   * Create a new plan
   */
  async createPlan(request: CreatePlanRequest): Promise<Plan> {
    const response = await apiClient.post<ApiResponse<Plan>>(
      "/api/plans",
      request
    );
    return response.data!;
  }

  /**
   * Get a specific plan by ID
   */
  async getPlan(planId: string): Promise<Plan> {
    const response = await apiClient.get<ApiResponse<Plan>>(
      `/api/plans/${planId}`
    );
    return response.data!;
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
   * Get plan statistics
   */
  async getPlanStatistics(): Promise<PlanStatistics> {
    const response = await apiClient.get<ApiResponse<PlanStatistics>>(
      "/api/analytics/plan-statistics"
    );
    return response.data!;
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
}

export const plansAPI = new PlansAPI();
export default plansAPI;
