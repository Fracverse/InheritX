import { describe, it, expect, beforeEach } from "vitest";
import { useAppStore } from "@/store/index";

// Reset store state between tests
beforeEach(() => {
  useAppStore.setState({
    address: null,
    selectedWalletId: null,
    isConnecting: false,
    isModalOpen: false,
    kycStatus: "pending",
    isKYCModalOpen: false,
    isSubmitting: false,
    plans: [],
    isLoadingPlans: false,
    plansError: null,
    lending: null,
    isLoadingLending: false,
    lendingError: null,
    locale: "en",
  });
});

describe("AppStore — Wallet slice", () => {
  it("sets wallet address", () => {
    useAppStore.getState().setAddress("GXYZ123");
    expect(useAppStore.getState().address).toBe("GXYZ123");
  });

  it("disconnects wallet", () => {
    useAppStore.getState().setAddress("GXYZ123");
    useAppStore.getState().disconnect();
    expect(useAppStore.getState().address).toBeNull();
    expect(useAppStore.getState().selectedWalletId).toBeNull();
  });

  it("toggles modal", () => {
    useAppStore.getState().setIsModalOpen(true);
    expect(useAppStore.getState().isModalOpen).toBe(true);
    useAppStore.getState().setIsModalOpen(false);
    expect(useAppStore.getState().isModalOpen).toBe(false);
  });
});

describe("AppStore — KYC slice", () => {
  it("updates KYC status", () => {
    useAppStore.getState().setKycStatus("approved");
    expect(useAppStore.getState().kycStatus).toBe("approved");
  });

  it("toggles KYC modal", () => {
    useAppStore.getState().setIsKYCModalOpen(true);
    expect(useAppStore.getState().isKYCModalOpen).toBe(true);
  });
});

describe("AppStore — Plans slice", () => {
  const mockPlan = {
    id: "001",
    name: "Test Plan",
    uniqueId: "uid_001",
    assets: "2 ETH",
    beneficiaryCount: 2,
    trigger: "INACTIVITY",
    status: "ACTIVE" as const,
  };

  it("adds a plan", () => {
    useAppStore.getState().addPlan(mockPlan);
    expect(useAppStore.getState().plans).toHaveLength(1);
    expect(useAppStore.getState().plans[0].id).toBe("001");
  });

  it("removes a plan", () => {
    useAppStore.getState().addPlan(mockPlan);
    useAppStore.getState().removePlan("001");
    expect(useAppStore.getState().plans).toHaveLength(0);
  });

  it("sets plans array", () => {
    useAppStore.getState().setPlans([mockPlan, { ...mockPlan, id: "002" }]);
    expect(useAppStore.getState().plans).toHaveLength(2);
  });

  it("tracks loading state", () => {
    useAppStore.getState().setIsLoadingPlans(true);
    expect(useAppStore.getState().isLoadingPlans).toBe(true);
  });

  it("tracks error state", () => {
    useAppStore.getState().setPlansError("Failed to load");
    expect(useAppStore.getState().plansError).toBe("Failed to load");
  });
});

describe("AppStore — Lending slice", () => {
  const mockLending = {
    totalDeposits: "12500000",
    totalBorrowed: "8750000",
    utilizationRate: 70,
    currentApy: 8.45,
    userShares: "5240",
    userBalance: "5240",
    totalEarnings: "142.50",
  };

  it("sets lending data", () => {
    useAppStore.getState().setLending(mockLending);
    expect(useAppStore.getState().lending?.currentApy).toBe(8.45);
  });

  it("tracks lending loading state", () => {
    useAppStore.getState().setIsLoadingLending(true);
    expect(useAppStore.getState().isLoadingLending).toBe(true);
  });
});

describe("AppStore — UI slice", () => {
  it("changes locale", () => {
    useAppStore.getState().setLocale("fr");
    expect(useAppStore.getState().locale).toBe("fr");
  });
});
