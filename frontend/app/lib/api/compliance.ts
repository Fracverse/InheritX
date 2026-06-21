/**
 * Compliance API Service
 * Handles compliance alerts, risk scoring, sanctions screening, and risk overrides.
 */

import { ApiClient, ApiResponse } from "./client";

export type ComplianceSeverity = "low" | "medium" | "high" | "critical";
export type RiskLevel = "low" | "medium" | "high" | "critical";
export type AlertStatus = "open" | "investigating" | "resolved";

export interface ComplianceAlert {
  id: string;
  type: "velocity" | "volume" | "flagged_transaction";
  address: string;
  user_id?: string;
  transaction_hash?: string;
  amount?: number;
  asset_code?: string;
  event_count?: number;
  threshold?: number;
  window_minutes?: number;
  severity: ComplianceSeverity;
  status: AlertStatus;
  reason: string;
  created_at: string;
  resolved_at?: string;
  metadata?: Record<string, unknown>;
}

export interface RiskFactor {
  label: string;
  impact: "positive" | "neutral" | "negative";
  score: number;
  description: string;
}

export interface RiskScore {
  address: string;
  score: number;
  level: RiskLevel;
  factors: RiskFactor[];
  last_evaluated_at: string;
  override?: RiskOverride;
}

export interface RiskOverride {
  id?: string;
  address: string;
  score: number;
  level: RiskLevel;
  justification: string;
  admin_id?: string;
  created_at?: string;
}

export interface SanctionsCheck {
  address: string;
  is_flagged: boolean;
  status: "clear" | "flagged" | "review";
  lists: string[];
  match_score?: number;
  checked_at: string;
  recommendation: string;
}

export interface RiskOverrideRequest {
  address: string;
  score: number;
  level: RiskLevel;
  justification: string;
}

type ComplianceApiResponse<T> = ApiResponse<T> | T;

function unwrapData<T>(response: ComplianceApiResponse<T>): T {
  if (
    response &&
    typeof response === "object" &&
    "data" in response &&
    (response as ApiResponse<T>).data !== undefined
  ) {
    return (response as ApiResponse<T>).data as T;
  }

  return response as T;
}

function ensureArray<T>(value: T[] | { alerts?: T[]; data?: T[] } | null | undefined): T[] {
  if (Array.isArray(value)) {
    return value;
  }

  if (value && Array.isArray(value.alerts)) {
    return value.alerts;
  }

  if (value && Array.isArray(value.data)) {
    return value.data;
  }

  return [];
}

export class ComplianceAPI {
  private client: ApiClient;

  constructor(baseUrl: string = "", getAuthToken?: () => string | null) {
    this.client = new ApiClient(baseUrl, getAuthToken);
  }

  async getVelocityAlerts(): Promise<ComplianceAlert[]> {
    const response = await this.client.get<
      ComplianceApiResponse<ComplianceAlert[] | { alerts?: ComplianceAlert[]; data?: ComplianceAlert[] }>
    >("/api/compliance/velocity-alerts");
    return ensureArray(unwrapData(response));
  }

  async getVolumeAlerts(): Promise<ComplianceAlert[]> {
    const response = await this.client.get<
      ComplianceApiResponse<ComplianceAlert[] | { alerts?: ComplianceAlert[]; data?: ComplianceAlert[] }>
    >("/api/compliance/volume-alerts");
    return ensureArray(unwrapData(response));
  }

  async getRiskScore(address: string): Promise<RiskScore> {
    const response = await this.client.get<ComplianceApiResponse<RiskScore>>(
      `/api/compliance/risk-score/${encodeURIComponent(address)}`,
    );
    return unwrapData(response);
  }

  async overrideRisk(payload: RiskOverrideRequest): Promise<RiskOverride> {
    const response = await this.client.post<ComplianceApiResponse<RiskOverride>>(
      "/api/compliance/risk-override",
      payload,
    );
    return unwrapData(response);
  }

  async checkSanctions(address: string): Promise<SanctionsCheck> {
    const response = await this.client.get<ComplianceApiResponse<SanctionsCheck>>(
      `/api/compliance/sanctions-check/${encodeURIComponent(address)}`,
    );
    return unwrapData(response);
  }
}

export function createComplianceAPI(
  getAuthToken: () => string | null = () => {
    if (typeof window === "undefined") {
      return null;
    }

    return localStorage.getItem("adminToken") || localStorage.getItem("auth_token");
  },
): ComplianceAPI {
  return new ComplianceAPI("", getAuthToken);
}

export const complianceAPI = createComplianceAPI();
export default complianceAPI;
