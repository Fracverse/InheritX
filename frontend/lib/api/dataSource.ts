import { plansAPI, type Plan } from "../../app/lib/api/plans";
import { kycAPI, type KYCResponse } from "../../app/lib/api/kyc";
import { mockStore } from "../mockStore";

export const useMockData = process.env.NEXT_PUBLIC_USE_MOCK_DATA === "true";

export async function getPlans(ownerAddress?: string): Promise<Plan[]> {
  if (useMockData) {
    const plans = mockStore.getPlans();
    return ownerAddress ? plans.filter((plan) => plan.owner_address === ownerAddress) : plans;
  }

  if (!ownerAddress) return [];
  return plansAPI.getPlansByOwner(ownerAddress);
}

export async function getPlan(planId: string): Promise<Plan | null> {
  if (useMockData) return mockStore.getPlan(planId);

  return plansAPI.getPlan(planId);
}

export async function getKYCStatus(walletAddress?: string): Promise<KYCResponse> {
  if (useMockData) {
    return {
      wallet_address: walletAddress ?? "demo-wallet",
      kyc_status: mockStore.getKYCStatus() as KYCResponse["kyc_status"],
      submitted_at: new Date().toISOString(),
    };
  }

  if (!walletAddress) throw new Error("Connect a wallet to load KYC status.");
  return kycAPI.getKYCStatus(walletAddress);
}