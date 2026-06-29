import type { StellarWalletsKit } from "@creit.tech/stellar-wallets-kit";
import type {
  CreatePlanRequest,
  PlanBeneficiaryRequest,
} from "@/app/lib/api/inheritance";

const SECONDS_PER_DAY = 86_400;
const DEFAULT_YIELD_RATE_BPS = 500;

export interface CreatePlanContractInput {
  owner: string;
  token: string;
  amount: number;
  beneficiaries: PlanBeneficiaryRequest[];
  gracePeriodDays: number;
  earnYield: boolean;
  timelockDays: number;
}

export interface CreatePlanContractResult {
  request: CreatePlanRequest;
  unsignedTransactionXdr: string;
  signedTransactionXdr?: string;
}

export function buildCreatePlanRequest(input: CreatePlanContractInput): CreatePlanRequest {
  const gracePeriod = input.gracePeriodDays * SECONDS_PER_DAY;

  return {
    owner: input.owner,
    token: input.token,
    amount: input.amount,
    beneficiaries: input.beneficiaries,
    last_ping: Math.floor(Date.now() / 1000),
    grace_period: gracePeriod,
    earn_yield: input.earnYield,
    yield_rate_bps: input.earnYield ? DEFAULT_YIELD_RATE_BPS : 0,
    is_active: true,
  };
}

function encodeBase64(value: string): string {
  if (typeof window !== "undefined" && typeof window.btoa === "function") {
    return window.btoa(value);
  }

  return Buffer.from(value, "utf-8").toString("base64");
}

function buildUnsignedCreatePlanXdr(input: CreatePlanContractInput): string {
  const payload = {
    contract: process.env.NEXT_PUBLIC_INHERITANCE_CONTRACT_ID ?? "inheritance-contract",
    method: "create_plan",
    owner: input.owner,
    token: input.token,
    amount: input.amount,
    beneficiaries: input.beneficiaries,
    grace_period: input.gracePeriodDays * SECONDS_PER_DAY,
    earn_yield: input.earnYield,
    yield_rate_bps: input.earnYield ? DEFAULT_YIELD_RATE_BPS : 0,
    timelock_duration: input.timelockDays * SECONDS_PER_DAY,
  };

  return `unsigned-xdr::create_plan::${encodeBase64(JSON.stringify(payload))}`;
}

export async function invokeCreatePlan(input: {
  contractInput: CreatePlanContractInput;
  kit: StellarWalletsKit | null;
  selectedWalletId: string | null;
}): Promise<CreatePlanContractResult> {
  const request = buildCreatePlanRequest(input.contractInput);
  const unsignedTransactionXdr = buildUnsignedCreatePlanXdr(input.contractInput);

  if (!input.kit || !input.selectedWalletId) {
    return { request, unsignedTransactionXdr };
  }

  const signed = await input.kit.signTransaction(unsignedTransactionXdr);
  return {
    request,
    unsignedTransactionXdr,
    signedTransactionXdr: signed.signedTxXdr,
  };
}

export const inheritanceContractService = {
  buildCreatePlanRequest,
  invokeCreatePlan,
};
