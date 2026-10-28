import { Plan } from "@/app/lib/api/plans";

// In-memory mock store for frontend demo mode
export interface MockBeneficiary {
  wallet_address: string;
  allocation_bps: number;
  fiat_anchor_info: string;
}

export interface MockPlan extends Plan {
  id: string;
  owner_address: string;
  token_address: string;
  amount: number;
  status: string;
  grace_period_seconds: number;
  earn_yield: boolean;
  yield_rate_bps: number;
  accrued_yield: number;
  last_ping: number;
  created_at: string;
  beneficiaries: MockBeneficiary[];
}

const INITIAL_PLANS: MockPlan[] = [
  {
    id: "plan_01jm8v45px",
    user_id: "user_1",
    owner_address: "GDRT7YV3XP9P...8K2M",
    token_address: "USDC",
    amount: 32000,
    title: "USDC Family Plan",
    fee: 10,
    net_amount: 31990,
    status: "ACTIVE",
    grace_period_seconds: 2592000,
    earn_yield: true,
    yield_rate_bps: 450,
    accrued_yield: 42.50,
    last_ping: Math.floor(Date.now() / 1000) - 86400,
    created_at: "2026-01-15T08:30:00Z",
    updated_at: "2026-01-15T08:30:00Z",
    beneficiaries: [
      { wallet_address: "GDX74T...3P2X", allocation_bps: 6000, fiat_anchor_info: "anchor-ngn" },
      { wallet_address: "GBN23P...8K9T", allocation_bps: 4000, fiat_anchor_info: "anchor-kes" },
    ],
  },
  {
    id: "plan_02km9v56py",
    user_id: "user_1",
    owner_address: "GCMW9AJX2M8K...1R4T",
    token_address: "USDC",
    amount: 50000,
    title: "Primary Savings",
    fee: 15,
    net_amount: 49985,
    status: "ACTIVE",
    grace_period_seconds: 5184000,
    earn_yield: true,
    yield_rate_bps: 500,
    accrued_yield: 184.20,
    last_ping: Math.floor(Date.now() / 1000) - 3600,
    created_at: "2026-02-10T14:00:00Z",
    updated_at: "2026-02-10T14:00:00Z",
    beneficiaries: [
      { wallet_address: "GBZV2I...1H4J", allocation_bps: 10000, fiat_anchor_info: "anchor-brl" },
    ],
  },
  {
    id: "plan_03lm1v23pz",
    user_id: "user_1",
    owner_address: "GCMW9AJX2M8K...1R4T",
    token_address: "EURC",
    amount: 15000,
    title: "EURC Inheritance",
    fee: 5,
    net_amount: 14995,
    status: "TRIGGERED",
    grace_period_seconds: 604800,
    earn_yield: false,
    yield_rate_bps: 0,
    accrued_yield: 0,
    last_ping: Math.floor(Date.now() / 1000) - 1209600,
    created_at: "2026-03-01T10:00:00Z",
    updated_at: "2026-03-01T10:00:00Z",
    beneficiaries: [
      { wallet_address: "GBXP2Z...5N7Q", allocation_bps: 5000, fiat_anchor_info: "anchor-eur" },
      { wallet_address: "GDKL3B...6V8W", allocation_bps: 5000, fiat_anchor_info: "anchor-ngn" },
    ],
  },
];

// Helper to persist mock state in window/client memory
const isClient = typeof window !== "undefined";

export const mockStore = {
  getPlans(): MockPlan[] {
    if (!isClient) return INITIAL_PLANS;
    const stored = localStorage.getItem("inheritx_mock_plans");
    if (!stored) {
      localStorage.setItem("inheritx_mock_plans", JSON.stringify(INITIAL_PLANS));
      return INITIAL_PLANS;
    }
    return JSON.parse(stored);
  },

  getPlan(id: string): MockPlan | null {
    return this.getPlans().find((p) => p.id === id) || null;
  },

  createPlan(plan: Partial<MockPlan>): MockPlan {
    const plans = this.getPlans();
    const newPlan: MockPlan = {
      id: `plan_${globalThis.crypto.randomUUID()}`,
      user_id: "user_logged_in",
      owner_address: plan.owner_address || "GDRT7YV3XP9P...8K2M",
      token_address: plan.token_address || "USDC",
      amount: plan.amount || 0,
      title: plan.title || `${plan.token_address} Vault Plan`,
      fee: 10,
      net_amount: (plan.amount || 0) - 10,
      status: "ACTIVE",
      grace_period_seconds: plan.grace_period_seconds || 2592000,
      earn_yield: plan.earn_yield || false,
      yield_rate_bps: plan.yield_rate_bps || 0,
      accrued_yield: 0,
      last_ping: Math.floor(Date.now() / 1000),
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
      beneficiaries: plan.beneficiaries || [],
    };
    plans.push(newPlan);
    localStorage.setItem("inheritx_mock_plans", JSON.stringify(plans));
    return newPlan;
  },

  updatePlan(id: string, updates: Partial<MockPlan>): MockPlan | null {
    const plans = this.getPlans();
    const index = plans.findIndex((p) => p.id === id);
    if (index === -1) return null;
    const updated = { ...plans[index], ...updates, updated_at: new Date().toISOString() };
    plans[index] = updated;
    localStorage.setItem("inheritx_mock_plans", JSON.stringify(plans));
    return updated;
  },

  pingPlan(id: string): boolean {
    const plan = this.getPlan(id);
    if (!plan) return false;
    this.updatePlan(id, { last_ping: Math.floor(Date.now() / 1000) });
    return true;
  },

  getKYCStatus(): string {
    if (!isClient) return "approved";
    return localStorage.getItem("inheritx_mock_kyc") || "approved";
  },

  setKYCStatus(status: string) {
    if (!isClient) return;
    localStorage.setItem("inheritx_mock_kyc", status);
  }
};
