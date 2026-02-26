import { Claim, Activity, PlanSummaryData } from "../types";

export const MOCK_CLAIMS: Claim[] = [
  {
    id: "001",
    planName: "Plan Name",
    uniqueId: "Unique ID",
    assets: "2 ETH",
    beneficiaryCount: 3,
    trigger: "INACTIVITY (6 MONTHS)",
    status: "ACTIVE",
  },
];

export const MOCK_ACTIVITIES: Activity[] = [
  {
    id: 1,
    description: "Plan #001 Created (3 Beneficiaries, Inactivity Trigger Set)",
    timestamp: "12th August, 2025",
  },
  {
    id: 2,
    description: "Guardian Added To Plan #002",
    timestamp: "12th August, 2025",
  },
  {
    id: 3,
    description: "Plan #001 Status Changed To Active",
    timestamp: "12th August, 2025",
  },
  {
    id: 4,
    description: "1 NFC Converted",
    timestamp: "12th August, 2025",
  },
];

export const MOCK_PLAN_SUMMARY: PlanSummaryData = {
  planName: "My first daughter",
  description: "This is an inheritance for my babygirl. My first daughter",
  beneficiary: "John Johnson",
  beneficiaryEmail: "jjohnson@gmail.com",
  walletAddress: "0xajoer....apro",
  executeOn: "16/04/2027",
  assets: ["Tokens", "NFTs", "Real-World Assets"],
};
