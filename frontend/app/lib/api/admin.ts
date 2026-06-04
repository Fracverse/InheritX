/**
 * Admin API Service
 * Handles all admin-related API calls
 */

import { apiClient, ApiResponse, PaginatedResponse } from "./client";

export interface AdminMetrics {
  total_revenue: number;
  total_plans: number;
  total_claims: number;
  active_plans: number;
  total_users: number;
}

export interface KycStatus {
  user_id: string;
  status: string;
  reviewed_by?: string;
  reviewed_at?: string;
  created_at: string;
}

export interface AuditLog {
  id: string;
  user_id?: string;
  admin_id?: string;
  action: string;
  entity_id?: string;
  entity_type?: string;
  timestamp: string;
  metadata?: any;
}

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
  beneficiary_name?: string;
  bank_name?: string;
  bank_account_number?: string;
  currency_preference?: string;
  created_at: string;
  updated_at: string;
}

export interface InsuranceFund {
  id: string;
  name: string;
  asset_code: string;
  balance: number;
  total_deposits: number;
  total_withdrawals: number;
  total_claims_paid: number;
  reserve_ratio: number;
  health_status: string;
  created_at: string;
  updated_at: string;
}

export interface InsuranceFundDashboard {
  fund: InsuranceFund;
  metrics: {
    current_balance: number;
    total_claims: number;
    pending_claims: number;
    approved_claims: number;
    rejected_claims: number;
    total_claims_amount: number;
    reserve_ratio: number;
    health_status: string;
  };
  recent_transactions: any[];
}

export class AdminAPI {
  /**
   * Get admin dashboard metrics
   */
  async getMetrics(): Promise<AdminMetrics> {
    const response = await apiClient.get<ApiResponse<AdminMetrics>>(
      "/api/admin/metrics"
    );
    return response.data!;
  }

  /**
   * Get all audit logs with pagination
   */
  async getAuditLogs(
    page: number = 1,
    limit: number = 20
  ): Promise<PaginatedResponse<AuditLog>> {
    return apiClient.get<PaginatedResponse<AuditLog>>(
      `/api/admin/logs?page=${page}&limit=${limit}`
    );
  }

  /**
   * Get KYC status for a user
   */
  async getKycStatus(userId: string): Promise<KycStatus> {
    const response = await apiClient.get<KycStatus>(
      `/api/admin/kyc/${userId}`
    );
    return response;
  }

  /**
   * Approve KYC for a user
   */
  async approveKyc(userId: string): Promise<KycStatus> {
    return apiClient.post<KycStatus>("/api/admin/kyc/approve", {
      user_id: userId,
    });
  }

  /**
   * Reject KYC for a user
   */
  async rejectKyc(userId: string): Promise<KycStatus> {
    return apiClient.post<KycStatus>("/api/admin/kyc/reject", {
      user_id: userId,
    });
  }

  /**
   * Get all plans (admin view)
   */
  async getAllPlans(
    page: number = 1,
    limit: number = 20
  ): Promise<PaginatedResponse<Plan>> {
    return apiClient.get<PaginatedResponse<Plan>>(
      `/api/admin/plans/due-for-claim?page=${page}&limit=${limit}`
    );
  }

  /**
   * Get insurance fund dashboard
   */
  async getInsuranceFundDashboard(): Promise<InsuranceFundDashboard> {
    const response = await apiClient.get<ApiResponse<InsuranceFundDashboard>>(
      "/api/admin/insurance-fund"
    );
    return response.data!;
  }

  /**
   * Get all insurance funds
   */
  async getAllInsuranceFunds(): Promise<InsuranceFund[]> {
    const response = await apiClient.get<ApiResponse<InsuranceFund[]>>(
      "/api/admin/insurance-funds"
    );
    return response.data!;
  }

  /**
   * Pause a plan
   */
  async pausePlan(planId: string, reason: string): Promise<any> {
    return apiClient.post("/api/admin/emergency/pause", {
      plan_id: planId,
      reason,
    });
  }

  /**
   * Unpause a plan
   */
  async unpausePlan(planId: string): Promise<any> {
    return apiClient.post("/api/admin/emergency/unpause", {
      plan_id: planId,
    });
  }

  /**
   * Set risk override for a plan
   */
  async setRiskOverride(
    planId: string,
    enabled: boolean,
    reason: string
  ): Promise<any> {
    return apiClient.post("/api/admin/emergency/risk-override", {
      plan_id: planId,
      enabled,
      reason,
    });
  }
}

export const adminAPI = new AdminAPI();
export default adminAPI;
