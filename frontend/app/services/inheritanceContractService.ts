import { apiClient, ApiError } from "@/app/lib/api/client";
import {
  buildContractPauseTransaction,
  buildCreateInheritancePlanTransaction,
  describeContractError,
  ContractSimulationError,
  readContractPaused,
  type ContractPauseAction,
} from "@/app/lib/stellar/sorobanClient";

export { ContractSimulationError };
import { toAtomicAmount, type ContractBeneficiaryInput, type DistributionMethod } from "@/app/lib/stellar/contractParams";
import { stellarExpertTxUrl } from "@/app/lib/stellar/network";

export type { ContractBeneficiaryInput, DistributionMethod };

export interface CreateInheritancePlanContractInput {
  owner: string;
  token: string;
  planName: string;
  description: string;
  /** Deposit amount as a plain decimal string, e.g. "150.5". */
  amount: string;
  distributionMethod: DistributionMethod;
  beneficiaries: ContractBeneficiaryInput[];
  isLendable: boolean;
}

/** Wallet-provided signer. Matches `WalletContext.signTransaction`. */
export type TransactionSigner = (
  xdr: string,
  opts?: { networkPassphrase?: string; address?: string }
) => Promise<{ signedTxXdr: string }>;

export type CreatePlanStage =
  | "building"
  | "awaiting-signature"
  | "submitting"
  | "confirmed";

export interface CreatePlanProgress {
  stage: CreatePlanStage;
}

export interface SubmitTransactionResult {
  hash: string;
  successful: boolean;
  raw: Record<string, unknown>;
}

export class TransactionSubmissionError extends Error {
  constructor(
    message: string,
    public readonly raw?: unknown
  ) {
    super(message);
    this.name = "TransactionSubmissionError";
  }
}

export interface InvokeCreateInheritancePlanResult {
  unsignedTransactionXdr: string;
  signedTransactionXdr: string;
  submission: SubmitTransactionResult;
  explorerUrl: string;
}

/** Builds the ready-to-sign XDR for `create_inheritance_plan` from panel input. */
export async function buildCreateInheritancePlanXdr(
  input: CreateInheritancePlanContractInput
): Promise<{ unsignedTransactionXdr: string; networkPassphrase: string }> {
  return buildCreateInheritancePlanTransaction({
    owner: input.owner,
    token: input.token,
    planName: input.planName,
    description: input.description,
    totalAmountAtomic: toAtomicAmount(input.amount),
    distributionMethod: input.distributionMethod,
    beneficiaries: input.beneficiaries,
    isLendable: input.isLendable,
  });
}

/** Submits an already-signed transaction envelope to the backend's Horizon relay. */
export async function submitSignedTransaction(
  signedTransactionXdr: string
): Promise<SubmitTransactionResult> {
  try {
    const response = await apiClient.post<Record<string, unknown>>(
      "/api/transactions/submit",
      { xdr: signedTransactionXdr }
    );

    const hash = typeof response.hash === "string" ? response.hash : "";
    const successful = response.successful === true;

    if (!hash) {
      throw new TransactionSubmissionError(
        "The network accepted the request but returned no transaction hash.",
        response
      );
    }

    if (successful && typeof window !== "undefined") {
      window.dispatchEvent(
        new CustomEvent("stellar:transaction-confirmed", { detail: { hash } })
      );
    }

    return { hash, successful, raw: response };
  } catch (error) {
    if (error instanceof ApiError) {
      const extras = (error.response as { extras?: { result_codes?: unknown } } | undefined)
        ?.extras;
      const detail = extras?.result_codes
        ? JSON.stringify(extras.result_codes)
        : error.message;
      throw new TransactionSubmissionError(
        `Transaction was rejected by the network: ${detail}`,
        error.response
      );
    }
    throw error;
  }
}

/**
 * Full on-chain plan creation flow: build + simulate the transaction, ask the
 * connected wallet to sign it, then relay the signed envelope through the
 * backend's `/api/transactions/submit` endpoint. Reports progress via
 * `onProgress` so the UI can show a step-by-step status.
 */
export async function invokeCreateInheritancePlan(options: {
  contractInput: CreateInheritancePlanContractInput;
  walletAddress: string;
  signTransaction: TransactionSigner;
  onProgress?: (progress: CreatePlanProgress) => void;
}): Promise<InvokeCreateInheritancePlanResult> {
  const { contractInput, walletAddress, signTransaction, onProgress } = options;

  onProgress?.({ stage: "building" });
  const { unsignedTransactionXdr, networkPassphrase } = await buildCreateInheritancePlanXdr(
    contractInput
  );

  onProgress?.({ stage: "awaiting-signature" });
  let signed;
  try {
    signed = await signTransaction(unsignedTransactionXdr, {
      networkPassphrase,
      address: walletAddress,
    });
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new ContractSimulationError(
      message.toLowerCase().includes("declin") || message.toLowerCase().includes("reject")
        ? "Signature request was declined in the wallet."
        : describeContractError(message),
      error
    );
  }

  onProgress?.({ stage: "submitting" });
  const submission = await submitSignedTransaction(signed.signedTxXdr);

  onProgress?.({ stage: "confirmed" });
  return {
    unsignedTransactionXdr,
    signedTransactionXdr: signed.signedTxXdr,
    submission,
    explorerUrl: stellarExpertTxUrl(submission.hash),
  };
}

export type { ContractPauseAction };

export interface InvokeContractPauseResult {
  submission: SubmitTransactionResult;
  explorerUrl: string;
}

/**
 * Emergency pause / unpause of the inheritance contract by an admin: build +
 * simulate `pause(admin)` / `unpause(admin)`, have the admin's wallet sign it,
 * and relay it through `/api/transactions/submit`. Same stages as plan creation.
 */
export async function invokeContractPause(options: {
  admin: string;
  action: ContractPauseAction;
  signTransaction: TransactionSigner;
  onProgress?: (progress: CreatePlanProgress) => void;
}): Promise<InvokeContractPauseResult> {
  const { admin, action, signTransaction, onProgress } = options;

  onProgress?.({ stage: "building" });
  const { unsignedTransactionXdr, networkPassphrase } = await buildContractPauseTransaction(
    admin,
    action
  );

  onProgress?.({ stage: "awaiting-signature" });
  let signed;
  try {
    signed = await signTransaction(unsignedTransactionXdr, { networkPassphrase, address: admin });
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new ContractSimulationError(
      message.toLowerCase().includes("declin") || message.toLowerCase().includes("reject")
        ? "Signature request was declined in the wallet."
        : message,
      error
    );
  }

  onProgress?.({ stage: "submitting" });
  const submission = await submitSignedTransaction(signed.signedTxXdr);
  if (!submission.successful) {
    throw new TransactionSubmissionError(
      `The ${action} transaction was submitted but did not succeed.`,
      submission.raw
    );
  }

  onProgress?.({ stage: "confirmed" });
  return { submission, explorerUrl: stellarExpertTxUrl(submission.hash) };
}

export { readContractPaused };

export const inheritanceContractService = {
  buildCreateInheritancePlanXdr,
  submitSignedTransaction,
  invokeCreateInheritancePlan,
  invokeContractPause,
  readContractPaused,
};
