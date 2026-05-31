/**
 * API Services Index
 * Central export point for all API services
 */

export { apiClient, ApiClient, ApiError } from "./client";
export type { ApiResponse, PaginatedResponse } from "./client";

export { adminAPI, AdminAPI } from "./admin";
export type {
  AdminMetrics,
  KycStatus,
  AuditLog,
  InsuranceFund,
  InsuranceFundDashboard,
} from "./admin";

export { plansAPI, PlansAPI } from "./plans";
export type {
  Plan,
  CreatePlanRequest,
  ClaimPlanRequest,
  PlanStatistics,
} from "./plans";

export { createLendingAPI, LendingAPI } from "./lending";
export type {
  PoolState,
  UserLendingData,
  LendingTransaction,
} from "./lending";

// Re-export commonly used services
export const api = {
  admin: adminAPI,
  plans: plansAPI,
  lending: createLendingAPI(),
};

export default api;
