/**
 * Centralized state management with Zustand
 * Install: npm install zustand
 */

import { create } from "zustand";
import { persist } from "zustand/middleware";
// ─── Realtime (WebSocket) helpers ─────────────────────────────────────────────
// Backend streams KycUpdateEvent / PlanStatusEvent over one broadcast socket
// (registered at /ws/kyc). Override with NEXT_PUBLIC_WS_URL or NEXT_PUBLIC_WS_PATH.

const WS_API_BASE = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080";
const WS_PATH = process.env.NEXT_PUBLIC_WS_PATH || "/ws/kyc";

function buildWsUrl(): string {
  const explicit = process.env.NEXT_PUBLIC_WS_URL;
  if (explicit) return explicit;
  return WS_API_BASE.replace(/^http/i, "ws").replace(/\/$/, "") + WS_PATH;
}

let socket: WebSocket | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
let reconnectAttempts = 0;
let shouldReconnect = false;
let watchedAddress: string | null = null;
let toastSeq = 0;


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

// ─── Genetic Privacy Slice ────────────────────────────────────────────────────

interface GeneticPrivacySlice {
  /** Last-saved genetic privacy settings, keyed by plan id. */
  geneticPrivacy: Record<string, PrivacySettings>;
  setGeneticPrivacySettings: (
    planId: string,
    settings: PrivacySettings,
  ) => void;
}

// ─── UI Slice ─────────────────────────────────────────────────────────────────

interface UISlice {
  locale: string;
  setLocale: (locale: string) => void;
}

// ─── Combined Store ───────────────────────────────────────────────────────────

// ─── Realtime (WebSocket) Slice ───────────────────────────────────────────────

export interface PlanStatusEvent {
  event_type: string;
  plan_id: string;
  status: string;
  message: string;
  tx_hash?: string;
  freeze_status?: string;
  recall_progress?: number;
  settlement_status?: string;
  remaining_loaned?: number;
}

export interface KycUpdateEvent {
  wallet_address: string;
  kyc_status: string;
  event_type: string;
}

export interface ToastItem {
  id: number;
  kind: "success" | "info" | "warning" | "error";
  title: string;
  message?: string;
}

interface RealtimeSlice {
  wsConnected: boolean;
  toasts: ToastItem[];
  lastPlanEvent: PlanStatusEvent | null;
  pushToast: (t: Omit<ToastItem, "id">) => void;
  dismissToast: (id: number) => void;
  connectWs: (walletAddress?: string | null) => void;
  disconnectWs: () => void;
}

type AppStore = WalletSlice &
  KYCSlice &
  PlansSlice &
  LendingSlice &
  UISlice &
  RealtimeSlice;


export const useAppStore = create<AppStore>()(
  persist(
    (set, get) => ({
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

      // Realtime (WebSocket)
      wsConnected: false,
      toasts: [],
      lastPlanEvent: null,
      pushToast: (t) => {
        const id = ++toastSeq;
        set((s) => ({ toasts: [...s.toasts.slice(-4), { ...t, id }] }));
        setTimeout(() => get().dismissToast(id), 6000);
      },
      dismissToast: (id) =>
        set((s) => ({ toasts: s.toasts.filter((x) => x.id !== id) })),
      connectWs: (walletAddress) => {
        if (typeof window === "undefined" || typeof WebSocket === "undefined") return;
        watchedAddress = walletAddress ?? null;
        if (
          socket &&
          (socket.readyState === WebSocket.OPEN ||
            socket.readyState === WebSocket.CONNECTING)
        ) {
          return;
        }
        shouldReconnect = true;
        const ws = new WebSocket(buildWsUrl());
        socket = ws;

        ws.onopen = () => {
          reconnectAttempts = 0;
          set({ wsConnected: true });
        };

        ws.onmessage = (msg) => {
          let data: unknown;
          try {
            data = JSON.parse(String(msg.data));
          } catch {
            return;
          }
          if (!data || typeof data !== "object") return;
          const ev = data as Record<string, unknown>;

          // KycUpdateEvent: { wallet_address, kyc_status, event_type }
          if (typeof ev.kyc_status === "string" && typeof ev.wallet_address === "string") {
            const mine = watchedAddress ?? get().address;
            if (!mine || mine.toLowerCase() !== ev.wallet_address.toLowerCase()) return;
            const next = ev.kyc_status.toLowerCase();
            if (next === "pending" || next === "submitted" || next === "approved" || next === "rejected") {
              set({ kycStatus: next as KYCStatus });
            }
            get().pushToast({
              kind: next === "approved" ? "success" : next === "rejected" ? "error" : "info",
              title: "KYC update",
              message: `Your KYC status is now ${next}.`,
            });
            return;
          }

          // PlanStatusEvent: { event_type, plan_id, status, message, ... }
          if (typeof ev.plan_id === "string" && typeof ev.status === "string") {
            const e = ev as unknown as PlanStatusEvent;
            const type = String(e.event_type ?? "").toLowerCase();
            set((s) => ({
              lastPlanEvent: e,
              plans: s.plans.map((p) =>
                p.id === e.plan_id ? { ...p, status: e.status as Plan["status"] } : p,
              ),
            }));
            get().pushToast({
              kind: /fail|error/.test(type)
                ? "error"
                : /complete|paid|payout|success/.test(type)
                  ? "success"
                  : "info",
              title: String(e.event_type ?? "Plan update").replace(/[._]/g, " "),
              message: e.message,
            });
          }
        };

        ws.onclose = () => {
          set({ wsConnected: false });
          socket = null;
          if (!shouldReconnect) return;
          const delay = Math.min(30000, 1000 * 2 ** reconnectAttempts++);
          reconnectTimer = setTimeout(() => get().connectWs(watchedAddress), delay);
        };

        ws.onerror = () => {
          ws.close();
        };
      },
      disconnectWs: () => {
        shouldReconnect = false;
        if (reconnectTimer) {
          clearTimeout(reconnectTimer);
          reconnectTimer = null;
        }
        if (socket) {
          socket.close();
          socket = null;
        }
        set({ wsConnected: false });
      },

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
        geneticPrivacy: state.geneticPrivacy,
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
