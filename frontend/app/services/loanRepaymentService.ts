import {
  buildRepayLoanTransaction,
  describeLendingError,
  fetchLoanPosition,
  fetchRepaymentQuote,
  type LoanPosition,
  type RepaymentQuote,
} from "@/app/lib/stellar/lendingClient";
import { ContractSimulationError } from "@/app/lib/stellar/sorobanClient";
import { stellarExpertTxUrl } from "@/app/lib/stellar/network";
import { useMockData } from "@/lib/api/dataSource";
import { mockLoans } from "@/lib/mockLoans";
import {
  submitSignedTransaction,
  type TransactionSigner,
} from "./inheritanceContractService";

export type { LoanPosition, RepaymentQuote };

export type RepayLoanStage = "building" | "awaiting-signature" | "submitting" | "confirmed";

export interface RepayLoanResult {
  hash: string;
  /** Stellar Expert link, or `null` in demo mode. */
  explorerUrl: string | null;
  /** Amount that was quoted immediately before signing. */
  quote: RepaymentQuote;
}

/** Loads the connected wallet's open loan (or `null`), from the contract or demo store. */
export async function getActiveLoan(borrower: string): Promise<LoanPosition | null> {
  if (useMockData) return mockLoans.getLoan(borrower);
  return fetchLoanPosition(borrower);
}

/** Loads the live payoff (principal + interest + late fee) for the borrower's loan. */
export async function getRepaymentQuote(loan: LoanPosition): Promise<RepaymentQuote> {
  if (useMockData) return mockLoans.getRepaymentQuote(loan.borrower);
  return fetchRepaymentQuote(loan.borrower, loan.principal);
}

/**
 * Full on-chain repayment: build + simulate `repay`, ask the wallet to sign
 * (authorising the nested SEP-41 token transfer of principal + interest),
 * then relay through the backend's `/api/transactions/submit`.
 */
export async function invokeRepayLoan(options: {
  loan: LoanPosition;
  quote: RepaymentQuote;
  signTransaction: TransactionSigner;
  onProgress?: (stage: RepayLoanStage) => void;
}): Promise<RepayLoanResult> {
  const { loan, quote, signTransaction, onProgress } = options;
  const borrower = loan.borrower;

  if (useMockData) {
    onProgress?.("building");
    onProgress?.("awaiting-signature");
    onProgress?.("submitting");
    const result = mockLoans.repay(borrower);
    onProgress?.("confirmed");
    return { hash: result.hash, explorerUrl: null, quote: result.quote };
  }

  onProgress?.("building");
  const { unsignedTransactionXdr, networkPassphrase } = await buildRepayLoanTransaction(borrower);

  onProgress?.("awaiting-signature");
  let signed;
  try {
    signed = await signTransaction(unsignedTransactionXdr, { networkPassphrase, address: borrower });
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    const lower = message.toLowerCase();
    throw new ContractSimulationError(
      lower.includes("declin") || lower.includes("reject")
        ? "Signature request was declined in the wallet."
        : describeLendingError(message),
      error
    );
  }

  onProgress?.("submitting");
  const submission = await submitSignedTransaction(signed.signedTxXdr);

  onProgress?.("confirmed");
  return { hash: submission.hash, explorerUrl: stellarExpertTxUrl(submission.hash), quote };
}

export const loanRepaymentService = { getActiveLoan, getRepaymentQuote, invokeRepayLoan };
