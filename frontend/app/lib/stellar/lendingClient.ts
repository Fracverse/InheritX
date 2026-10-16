import {
  Account,
  Address,
  BASE_FEE,
  Contract,
  TransactionBuilder,
  rpc,
  scValToNative,
  type xdr,
} from "@stellar/stellar-sdk";
import {
  getLendingContractId,
  SOROBAN_RPC_URL,
  STELLAR_NETWORK_PASSPHRASE,
  TRANSACTION_TIMEOUT_SECONDS,
} from "./network";
import { ContractSimulationError } from "./sorobanClient";

/**
 * Soroban client for the `lending-contract` loan lifecycle
 * (contracts/lending-contract/src/lib.rs): reading the caller's open loan and
 * its live payoff amount, and building the `repay` invocation.
 *
 * `repay(borrower)` settles principal + accrued interest + any late fee in a
 * single call: the contract pulls the loan asset from the borrower via the
 * SEP-41 token `transfer` (authorised by the borrower's signature on this
 * transaction) and returns the full collateral in the same invocation.
 */

/** Maps `LendingError` discriminants to user-facing text. */
const LENDING_ERROR_MESSAGES: Record<number, string> = {
  1: "The lending contract hasn't been initialized yet.",
  4: "The pool doesn't have enough liquidity for this action.",
  6: "This wallet has no open loan.",
  8: "The amount is invalid for this loan.",
  9: "The token transfer failed — check that your balance covers the full repayment.",
  10: "Wallet is not authorized to perform this action.",
  14: "Another lending operation is in progress. Try again in a moment.",
  18: "Loan not found.",
  24: "This lending pool is paused.",
  25: "That asset isn't supported by the lending pool.",
  33: "The lending contract is paused.",
};

export function describeLendingError(raw: string): string {
  const match = raw.match(/Error\(Contract,\s*#(\d+)\)/);
  if (match) {
    const code = Number(match[1]);
    return LENDING_ERROR_MESSAGES[code] ?? `Lending contract rejected the transaction (error #${code}).`;
  }
  return raw;
}

/** JS view of the contract's `LoanRecord`. Amounts are atomic token units. */
export interface LoanPosition {
  loanId: bigint;
  borrower: string;
  /** Asset that was borrowed and must be repaid. */
  asset: string;
  principal: bigint;
  collateralAmount: bigint;
  collateralToken: string;
  /** Unix seconds. */
  borrowTime: number;
  /** Unix seconds. */
  dueDate: number;
  interestRateBps: number;
}

/** Live payoff figures returned by the contract at the current ledger time. */
export interface RepaymentQuote {
  principal: bigint;
  interest: bigint;
  lateFee: bigint;
  total: bigint;
}

function getServer(): rpc.Server {
  return new rpc.Server(SOROBAN_RPC_URL, {
    allowHttp: SOROBAN_RPC_URL.startsWith("http://"),
  });
}

function requireContractId(): string {
  const contractId = getLendingContractId();
  if (!contractId) {
    throw new ContractSimulationError(
      "NEXT_PUBLIC_LENDING_CONTRACT_ID is not configured for this environment."
    );
  }
  return contractId;
}

/**
 * Runs a read-only contract call through simulation. Simulation does not
 * consume the source account's sequence number, so a zero-sequence
 * placeholder for the caller's own address is sufficient.
 */
async function simulateRead(
  server: rpc.Server,
  caller: string,
  method: string,
  ...args: xdr.ScVal[]
): Promise<xdr.ScVal | undefined> {
  const contract = new Contract(requireContractId());
  const tx = new TransactionBuilder(new Account(caller, "0"), {
    fee: BASE_FEE,
    networkPassphrase: STELLAR_NETWORK_PASSPHRASE,
  })
    .addOperation(contract.call(method, ...args))
    .setTimeout(TRANSACTION_TIMEOUT_SECONDS)
    .build();

  const sim = await server.simulateTransaction(tx);
  if (rpc.Api.isSimulationError(sim)) {
    throw new ContractSimulationError(describeLendingError(sim.error), sim);
  }
  return sim.result?.retval;
}

function toBigInt(value: unknown): bigint {
  if (typeof value === "bigint") return value;
  if (typeof value === "number") return BigInt(Math.trunc(value));
  if (typeof value === "string" && value !== "") return BigInt(value);
  return BigInt(0);
}

/** Maps a decoded `LoanRecord` (snake_case, from `scValToNative`) into a `LoanPosition`. */
export function parseLoanRecord(native: Record<string, unknown>): LoanPosition {
  return {
    loanId: toBigInt(native.loan_id),
    borrower: String(native.borrower),
    asset: String(native.asset),
    principal: toBigInt(native.principal),
    collateralAmount: toBigInt(native.collateral_amount),
    collateralToken: String(native.collateral_token),
    borrowTime: Number(native.borrow_time ?? 0),
    dueDate: Number(native.due_date ?? 0),
    interestRateBps: Number(native.interest_rate_bps ?? 0),
  };
}

/** Reads the borrower's open loan via `get_loan`, or `null` when there is none. */
export async function fetchLoanPosition(borrower: string): Promise<LoanPosition | null> {
  const server = getServer();
  const retval = await simulateRead(server, borrower, "get_loan", Address.fromString(borrower).toScVal());
  if (!retval) return null;
  const native = scValToNative(retval) as Record<string, unknown> | null | undefined;
  return native ? parseLoanRecord(native) : null;
}

/**
 * Reads the exact payoff the contract will charge right now: `get_repayment_amount`
 * (principal + interest + late fee) and `calculate_late_fee`, so interest can
 * be shown separately.
 */
export async function fetchRepaymentQuote(
  borrower: string,
  principal: bigint
): Promise<RepaymentQuote> {
  const server = getServer();
  const borrowerArg = Address.fromString(borrower).toScVal();
  const [totalVal, lateFeeVal] = await Promise.all([
    simulateRead(server, borrower, "get_repayment_amount", borrowerArg),
    simulateRead(server, borrower, "calculate_late_fee", borrowerArg),
  ]);
  const total = totalVal ? toBigInt(scValToNative(totalVal)) : BigInt(0);
  const lateFee = lateFeeVal ? toBigInt(scValToNative(lateFeeVal)) : BigInt(0);
  const interest = total - principal - lateFee;
  return {
    principal,
    interest: interest > BigInt(0) ? interest : BigInt(0),
    lateFee,
    total,
  };
}

export interface BuildRepayLoanTxResult {
  unsignedTransactionXdr: string;
  networkPassphrase: string;
}

/**
 * Builds, simulates and assembles `repay(borrower)` into a ready-to-sign XDR.
 * Simulation records the auth entries for the nested token `transfer`, and
 * surfaces failures such as an insufficient balance before the wallet prompts.
 */
export async function buildRepayLoanTransaction(borrower: string): Promise<BuildRepayLoanTxResult> {
  const contract = new Contract(requireContractId());
  const server = getServer();

  let sourceAccount;
  try {
    sourceAccount = await server.getAccount(borrower);
  } catch (error) {
    throw new ContractSimulationError(
      "Couldn't load your account from the Stellar network. Make sure the wallet is funded on this network.",
      error
    );
  }

  const transaction = new TransactionBuilder(sourceAccount, {
    fee: BASE_FEE,
    networkPassphrase: STELLAR_NETWORK_PASSPHRASE,
  })
    .addOperation(contract.call("repay", Address.fromString(borrower).toScVal()))
    .setTimeout(TRANSACTION_TIMEOUT_SECONDS)
    .build();

  let prepared;
  try {
    prepared = await server.prepareTransaction(transaction);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new ContractSimulationError(describeLendingError(message), error);
  }

  return {
    unsignedTransactionXdr: prepared.toXDR(),
    networkPassphrase: STELLAR_NETWORK_PASSPHRASE,
  };
}
