/**
 * API Services Index
 * Central export point for all API services
 */

export { apiClient, ApiClient, ApiError } from "./client";
export type { ApiResponse, PaginatedResponse } from "./client";

import { AdminAPI } from "./admin";
export { AdminAPI } from "./admin";
export type {
  AdminMetrics,
  KycStatus,
  AuditLog,
  InsuranceFund,
  InsuranceFundDashboard,
} from "./admin";

import { PlansAPI } from "./plans";
export { PlansAPI } from "./plans";
export type {
  Plan,
  Beneficiary,
  CreatePlanRequest,
  UpdatePlanRequest,
  ClaimPlanRequest,
  PlanStatistics,
} from "./plans";

import { createLendingAPI } from "./lending";
export { createLendingAPI, LendingAPI } from "./lending";
export type {
  PoolState,
  UserLendingData,
  LendingTransaction,
} from "./lending";

import { ComplianceAPI } from "./compliance";
export { ComplianceAPI, createComplianceAPI } from "./compliance";
export type {
  AlertStatus,
  ComplianceAlert,
  ComplianceSeverity,
  RiskFactor,
  RiskLevel,
  RiskOverride,
  RiskOverrideRequest,
  RiskScore,
  SanctionsCheck,
} from "./compliance";

// Create instances
const adminAPI = new AdminAPI();
const plansAPI = new PlansAPI();
const complianceAPI = new ComplianceAPI();

// Re-export commonly used services
export const api = {
  admin: adminAPI,
  plans: plansAPI,
  lending: createLendingAPI(),
  compliance: complianceAPI,
};

export default api;
