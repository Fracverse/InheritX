/**
 * Centralized state management with Zustand
 * Install: npm install zustand
 */

import { create } from "zustand";
import { persist } from "zustand/middleware";

// ─── Types ────────────────────────────────────────────────────────────────────

export type KYCStatus = "pending" | "submitted" | "approved" | "rejected";

export interface Plan {
  id: string;
  name: string;
  uniqueId: string;
  assets: string;
  beneficiaryCount: number;
  trigger: string;
  status: "ACTIVE" | "COMPLETED" | "PENDING" | "EXPIRED";
}

export interface LendingState {
  totalDeposits: string;
  totalBorrowed: string;
  utilizationRate: number;
  currentApy: number;
  userShares: string;
  userBalance: string;
  totalEarnings: string;
}

// ─── Wallet Slice ─────────────────────────────────────────────────────────────

interface WalletSlice {
  address: string | null;
  selectedWalletId: string | null;
  isConnecting: boolean;
  isModalOpen: boolean;
  setAddress: (address: string | null) => void;
  setSelectedWalletId: (id: string | null) => void;
  setIsConnecting: (v: boolean) => void;
  setIsModalOpen: (v: boolean) => void;
  disconnect: () => void;
}

// ─── KYC Slice ────────────────────────────────────────────────────────────────

interface KYCSlice {
  kycStatus: KYCStatus;
  isKYCModalOpen: boolean;
  isSubmitting: boolean;
  setKycStatus: (status: KYCStatus) => void;
  setIsKYCModalOpen: (v: boolean) => void;
  setIsSubmitting: (v: boolean) => void;
}

// ─── Plans Slice ──────────────────────────────────────────────────────────────

interface PlansSlice {
  plans: Plan[];
  isLoadingPlans: boolean;
  plansError: string | null;
  setPlans: (plans: Plan[]) => void;
  setIsLoadingPlans: (v: boolean) => void;
  setPlansError: (err: string | null) => void;
  addPlan: (plan: Plan) => void;
  removePlan: (id: string) => void;
}

// ─── Lending Slice ────────────────────────────────────────────────────────────

interface LendingSlice {
  lending: LendingState | null;
  isLoadingLending: boolean;
  lendingError: string | null;
  setLending: (data: LendingState) => void;
  setIsLoadingLending: (v: boolean) => void;
  setLendingError: (err: string | null) => void;
}

// ─── UI Slice ─────────────────────────────────────────────────────────────────

interface UISlice {
  locale: string;
  setLocale: (locale: string) => void;
}

// ─── Combined Store ───────────────────────────────────────────────────────────

type AppStore = WalletSlice & KYCSlice & PlansSlice & LendingSlice & UISlice;

export const useAppStore = create<AppStore>()(
  persist(
    (set) => ({
      // Wallet
      address: null,
      selectedWalletId: null,
      isConnecting: false,
      isModalOpen: false,
      setAddress: (address) => set({ address }),
      setSelectedWalletId: (id) => set({ selectedWalletId: id }),
      setIsConnecting: (v) => set({ isConnecting: v }),
      setIsModalOpen: (v) => set({ isModalOpen: v }),
      disconnect: () => set({ address: null, selectedWalletId: null }),

      // KYC
      kycStatus: "pending",
      isKYCModalOpen: false,
      isSubmitting: false,
      setKycStatus: (status) => set({ kycStatus: status }),
      setIsKYCModalOpen: (v) => set({ isKYCModalOpen: v }),
      setIsSubmitting: (v) => set({ isSubmitting: v }),

      // Plans
      plans: [],
      isLoadingPlans: false,
      plansError: null,
      setPlans: (plans) => set({ plans }),
      setIsLoadingPlans: (v) => set({ isLoadingPlans: v }),
      setPlansError: (err) => set({ plansError: err }),
      addPlan: (plan) => set((s) => ({ plans: [...s.plans, plan] })),
      removePlan: (id) =>
        set((s) => ({ plans: s.plans.filter((p) => p.id !== id) })),

      // Lending
      lending: null,
      isLoadingLending: false,
      lendingError: null,
      setLending: (data) => set({ lending: data }),
      setIsLoadingLending: (v) => set({ isLoadingLending: v }),
      setLendingError: (err) => set({ lendingError: err }),

      // UI
      locale: "en",
      setLocale: (locale) => set({ locale }),
    }),
    {
      name: "inheritx-store",
      // Only persist non-sensitive, non-transient state
      partialize: (state) => ({
        address: state.address,
        selectedWalletId: state.selectedWalletId,
        kycStatus: state.kycStatus,
        locale: state.locale,
      }),
    },
  ),
);

// ─── Selectors ────────────────────────────────────────────────────────────────

export const selectIsConnected = (s: AppStore) => !!s.address;
export const selectWallet = (s: AppStore) => ({
  address: s.address,
  selectedWalletId: s.selectedWalletId,
  isConnecting: s.isConnecting,
  isModalOpen: s.isModalOpen,
});
export const selectKYC = (s: AppStore) => ({
  kycStatus: s.kycStatus,
  isKYCModalOpen: s.isKYCModalOpen,
  isSubmitting: s.isSubmitting,
});
export const selectPlans = (s: AppStore) => ({
  plans: s.plans,
  isLoadingPlans: s.isLoadingPlans,
  plansError: s.plansError,
});
export const selectLending = (s: AppStore) => ({
  lending: s.lending,
  isLoadingLending: s.isLoadingLending,
  lendingError: s.lendingError,
});
