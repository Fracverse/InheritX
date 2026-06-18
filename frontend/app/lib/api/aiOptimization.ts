export interface AssetAllocation {
  assetSymbol: string;
  chain: string;
  currentPercentage: number;
  recommendedPercentage: number;
  adjustmentReason: string;
  expectedImpact: string;
}

export interface RiskMetrics {
  volatility: number;
  sharpeRatio: number;
  maxDrawdown: number;
  valueAtRisk: number;
}

export interface ProjectedOutcomes {
  estimatedValue1Year: number;
  estimatedValue5Year: number;
  estimatedValue10Year: number;
  riskMetrics: RiskMetrics;
}

export interface OptimizationRecommendation {
  id: string;
  planId: number;
  recommendedAllocations: AssetAllocation[];
  confidenceScore: number;
  expectedReturn: number;
  riskScore: number;
  reasoning: string;
  generatedAt: string;
  projectedOutcomes: ProjectedOutcomes;
}

export interface RecommendationResponse {
  status: "accepted" | "rejected";
  reason?: string;
  appliedAt: string;
}

export interface CustomProjection {
  allocations: AssetAllocation[];
  projectedOutcomes: ProjectedOutcomes;
  expectedReturn: number;
  riskScore: number;
}

export class AIOptimizationAPI {
  private baseUrl: string;
  private getAuthToken: () => string | null;

  constructor(baseUrl: string = "", getAuthToken: () => string | null) {
    this.baseUrl = baseUrl;
    this.getAuthToken = getAuthToken;
  }

  private async request<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
    const token = this.getAuthToken();
    const headers: Record<string, string> = {
      "Content-Type": "application/json",
      ...((options.headers as Record<string, string>) || {}),
    };
    if (token) headers["Authorization"] = `Bearer ${token}`;

    const response = await fetch(`${this.baseUrl}${endpoint}`, { ...options, headers });
    if (!response.ok) {
      const error = await response.json().catch(() => ({}));
      throw new Error(error.error || `Request failed with status ${response.status}`);
    }
    return response.json();
  }

  async getOptimizationRecommendation(planId: number): Promise<OptimizationRecommendation> {
    return this.request<OptimizationRecommendation>(`/api/ai/optimize/${planId}`);
  }

  async acceptRecommendation(recommendationId: string): Promise<RecommendationResponse> {
    return this.request<RecommendationResponse>(
      `/api/ai/recommendations/${recommendationId}/respond`,
      { method: "POST", body: JSON.stringify({ action: "accept" }) }
    );
  }

  async rejectRecommendation(recommendationId: string, reason: string): Promise<RecommendationResponse> {
    return this.request<RecommendationResponse>(
      `/api/ai/recommendations/${recommendationId}/respond`,
      { method: "POST", body: JSON.stringify({ action: "reject", reason }) }
    );
  }

  async getCustomProjection(planId: number, allocations: AssetAllocation[]): Promise<CustomProjection> {
    return this.request<CustomProjection>(`/api/ai/optimize/${planId}/custom`, {
      method: "POST",
      body: JSON.stringify({ allocations }),
    });
  }
}

export function createAIOptimizationAPI(
  getAuthToken: () => string | null = () =>
    typeof localStorage !== "undefined" ? localStorage.getItem("auth_token") : null
): AIOptimizationAPI {
  return new AIOptimizationAPI("", getAuthToken);
}

export default createAIOptimizationAPI;
