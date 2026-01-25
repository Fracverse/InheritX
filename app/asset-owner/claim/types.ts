export type ViewState = "list" | "form" | "summary" | "success" | "error";

export interface Claim {
  id: string;
  planName: string;
  uniqueId: string;
  assets: string;
  beneficiaryCount: number;
  trigger: string;
  status: string;
}

export interface Activity {
  id: number;
  description: string;
  timestamp: string;
}

export interface PlanSummaryData {
  planName: string;
  description: string;
  beneficiary: string;
  beneficiaryEmail: string;
  walletAddress: string;
  executeOn: string;
  assets: string[];
}

export interface ClaimFormData {
  beneficiaryName: string;
  beneficiaryEmail: string;
  claimCode: string[];
}
