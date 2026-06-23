import { Claim, Activity, PlanSummaryData } from "../types";

export const MOCK_CLAIMS: Claim[] = [
  {
    id: "001",
    planName: "Testnet testing",
    uniqueId: "GP-001",
    assets: "15,000 XLM & 4,500 USDC",
    beneficiaryCount: 2,
    trigger: "TRIGGERED (ON-CHAIN BYPASS)",
    status: "CLAIMABLE",
  },
  {
    id: "002",
    planName: "Family Trust Plan",
    uniqueId: "GP-002",
    assets: "2 ETH",
    beneficiaryCount: 3,
    trigger: "INACTIVITY (6 MONTHS)",
    status: "ACTIVE",
  },
];

export const MOCK_ACTIVITIES: Activity[] = [
  {
    id: 1,
    description: "Plan #001 Created (2 Beneficiaries, Inactivity Trigger Set)",
    timestamp: "12th August, 2025",
  },
  {
    id: 2,
    description: "Inheritance Trigger Authorized by Admin/Guardian",
    timestamp: "23rd June, 2026",
  },
  {
    id: 3,
    description: "Outstanding Soroban Loans Frozen and Recalled",
    timestamp: "23rd June, 2026",
  },
  {
    id: 4,
    description: "Collateral Settlement Completed & Bypassed Time Locks",
    timestamp: "23rd June, 2026",
  },
];

export const MOCK_PLAN_SUMMARY: PlanSummaryData = {
  planName: "Testnet testing",
  description: "Secure asset distribution plan with soroban pool integrations",
  beneficiary: "John Doe",
  beneficiaryEmail: "john@doe.com",
  walletAddress: "GDE2KZQ4QGJZ5Z5QW2Y4B7Y6Q5D3P9V8N7M6L5K4J3H2G1FTEST",
  executeOn: "Bypassed (Inheritance Trigger Active)",
  assets: ["Tokens: 15,000 XLM & 4,500 USDC", "Soroban Collateral Reserves"],
};
