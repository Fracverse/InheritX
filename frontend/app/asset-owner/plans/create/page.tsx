"use client";

import { KYCRequiredGuard } from "@/components/kyc/KYCRequiredGuard";
import { CrossChainWalletProvider } from "@/context/CrossChainWalletContext";
import { CreateInheritancePlanPanel } from "@/components/plans/CreateInheritancePlanPanel";

export default function CreatePlanPage() {
  return (
    <KYCRequiredGuard>
      <CrossChainWalletProvider>
        <CreateInheritancePlanPanel />
      </CrossChainWalletProvider>
    </KYCRequiredGuard>
  );
}
