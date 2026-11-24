/**
 * Admin API Service
 * Handles all admin-related API calls
 */

import { apiClient, ApiError, ApiResponse, PaginatedResponse } from "./client";
import type {
  AdminPlan,
  AdminUser,
  KYCStatus,
  UserStatus,
} from "@/lib/adminTypes";

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

/** User record as returned by the admin users endpoints. */
export interface AdminUserRecord {
  id: string;
  wallet_address: string;
  created_at: string;
  kyc_status: string;
  active_plans_count: number;
  /** May be serialised as a decimal string. */
  tvl: number | string;
  status: string;
}

/** Plan record as returned by the admin plans endpoints. */
export interface AdminPlanRecord {
  id: string;
  owner_address: string;
  token_address: string;
  /** May be serialised as a decimal string. */
  amount: number | string;
  yield_rate_bps: number;
  status: string;
  /** Unix epoch seconds. */
  last_ping: number;
  heirs_count: number;
}

/** Records fetched per request when loading a full admin list. */
const LIST_PAGE_SIZE = 100;
/** Upper bound on pages fetched, so a misbehaving `has_next` can't loop forever. */
const LIST_MAX_PAGES = 100;

/**
 * Raised when the server answers 2xx but the body is not a usable record.
 * The api client returns `{}` for empty/non-JSON 2xx responses, so without this
 * check such a response would look like a successful action.
 */
function malformedResponse(what: string): ApiError {
  return new ApiError(`The server returned an unexpected response for ${what}.`, 502);
}

function requireRecord<T extends { id: string }>(record: unknown, what: string): T {
  if (
    !record ||
    typeof record !== "object" ||
    typeof (record as { id?: unknown }).id !== "string"
  ) {
    throw malformedResponse(what);
  }
  return record as T;
}

const KYC_STATUSES: readonly KYCStatus[] = ["pending", "approved", "rejected"];

/** Maps a server KYC status to the UI's. "submitted" is awaiting admin review, i.e. pending. */
export function toKycStatus(value: unknown): KYCStatus {
  if (value === "submitted") return "pending";
  if (typeof value === "string" && KYC_STATUSES.includes(value as KYCStatus)) {
    return value as KYCStatus;
  }
  throw malformedResponse("KYC status");
}

function toUserStatus(value: unknown): UserStatus {
  if (value === "active" || value === "suspended") return value;
  throw malformedResponse("user status");
}

export function toAdminUser(raw: unknown): AdminUser {
  const r = requireRecord<AdminUserRecord>(raw, "a user");
  return {
    id: r.id,
    walletAddress: r.wallet_address,
    registrationDate: r.created_at,
    kycStatus: toKycStatus(r.kyc_status),
    activePlansCount: Number(r.active_plans_count),
    tvl: Number(r.tvl),
    status: toUserStatus(r.status),
  };
}

export function toAdminPlan(raw: unknown): AdminPlan {
  const r = requireRecord<AdminPlanRecord>(raw, "a plan");
  return {
    id: r.id,
    owner: r.owner_address,
    token: r.token_address,
    amount: Number(r.amount),
    yieldRateBps: Number(r.yield_rate_bps),
    status: r.status,
    heirs: Number(r.heirs_count),
    lastPing: new Date(Number(r.last_ping) * 1000).toISOString(),
  };
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

  /**
   * Fetch every page of an admin list endpoint and map each record.
   */
  private async fetchAllPages<T>(
    path: string,
    map: (record: unknown) => T
  ): Promise<T[]> {
    const items: T[] = [];
    for (let page = 1; page <= LIST_MAX_PAGES; page++) {
      const res = await apiClient.get<PaginatedResponse<unknown>>(
        `${path}?page=${page}&limit=${LIST_PAGE_SIZE}`
      );
      if (!res || !Array.isArray(res.data)) {
        throw malformedResponse("the list");
      }
      items.push(...res.data.map(map));
      if (!res.has_next) return items;
    }
    throw new ApiError("Too many records to load.", 502);
  }

  /**
   * List all users (admin view)
   */
  async listUsers(): Promise<AdminUser[]> {
    return this.fetchAllPages("/api/admin/users", toAdminUser);
  }

  /**
   * Approve KYC for a user; resolves to the KYC status the server now holds.
   */
  async approveUserKyc(userId: string): Promise<KYCStatus> {
    const res = await this.approveKyc(userId);
    return toKycStatus(res?.status);
  }

  /**
   * Suspend a user; resolves to the updated user.
   */
  async suspendUser(userId: string): Promise<AdminUser> {
    return toAdminUser(
      await apiClient.post(`/api/admin/users/${encodeURIComponent(userId)}/suspend`)
    );
  }

  /**
   * Lift a user's suspension; resolves to the updated user.
   */
  async unsuspendUser(userId: string): Promise<AdminUser> {
    return toAdminUser(
      await apiClient.post(`/api/admin/users/${encodeURIComponent(userId)}/unsuspend`)
    );
  }

  /**
   * Terminate all of a user's active plans; resolves to the updated user.
   */
  async terminateUserPlans(userId: string): Promise<AdminUser> {
    return toAdminUser(
      await apiClient.post(
        `/api/admin/users/${encodeURIComponent(userId)}/terminate-plans`
      )
    );
  }

  /**
   * List all plans (admin oversight view)
   */
  async listPlans(): Promise<AdminPlan[]> {
    return this.fetchAllPages("/api/admin/plans", toAdminPlan);
  }

  /**
   * Terminate a plan; resolves to the updated plan.
   */
  async terminatePlan(planId: string): Promise<AdminPlan> {
    return toAdminPlan(
      await apiClient.post(`/api/admin/plans/${encodeURIComponent(planId)}/terminate`)
    );
  }
}

export const adminAPI = new AdminAPI();
export default adminAPI;
