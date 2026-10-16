import { describe, expect, it, vi, beforeEach } from "vitest";

const getAccountMock = vi.fn();
const prepareTransactionMock = vi.fn();
const simulateTransactionMock = vi.fn();

vi.mock("@stellar/stellar-sdk", async () => {
  const actual = await vi.importActual<typeof import("@stellar/stellar-sdk")>("@stellar/stellar-sdk");
  return {
    ...actual,
    rpc: {
      ...actual.rpc,
      Server: vi.fn().mockImplementation(() => ({
        getAccount: getAccountMock,
        prepareTransaction: prepareTransactionMock,
        simulateTransaction: simulateTransactionMock,
      })),
    },
  };
});

import { Account, Address, StrKey, Transaction, nativeToScVal, xdr } from "@stellar/stellar-sdk";
import {
  buildRepayLoanTransaction,
  describeLendingError,
  fetchLoanPosition,
  fetchRepaymentQuote,
} from "@/app/lib/stellar/lendingClient";
import { ContractSimulationError } from "@/app/lib/stellar/sorobanClient";
import { STELLAR_NETWORK_PASSPHRASE } from "@/app/lib/stellar/network";

const BORROWER = StrKey.encodeEd25519PublicKey(Buffer.alloc(32, 4));
const ASSET = StrKey.encodeContract(Buffer.alloc(32, 1));
const COLLATERAL = StrKey.encodeContract(Buffer.alloc(32, 2));
const LENDING_ID = StrKey.encodeContract(Buffer.alloc(32, 3));

function entry(key: string, val: xdr.ScVal) {
  return new xdr.ScMapEntry({ key: xdr.ScVal.scvSymbol(key), val });
}

/** Encodes a `LoanRecord` the way the contract's `#[contracttype]` derive does. */
function loanRecordScVal(): xdr.ScVal {
  return xdr.ScVal.scvMap([
    entry("asset", Address.fromString(ASSET).toScVal()),
    entry("borrow_time", nativeToScVal(BigInt(1_700_000_000), { type: "u64" })),
    entry("borrower", Address.fromString(BORROWER).toScVal()),
    entry("collateral_amount", nativeToScVal(BigInt(15_000_0000000), { type: "u64" })),
    entry("collateral_token", Address.fromString(COLLATERAL).toScVal()),
    entry("due_date", nativeToScVal(BigInt(1_702_592_000), { type: "u64" })),
    entry("interest_rate_bps", xdr.ScVal.scvU32(500)),
    entry("loan_id", nativeToScVal(BigInt(7), { type: "u64" })),
    entry("principal", nativeToScVal(BigInt(9_000_0000000), { type: "u64" })),
  ]);
}

beforeEach(() => {
  vi.clearAllMocks();
  process.env.NEXT_PUBLIC_LENDING_CONTRACT_ID = LENDING_ID;
});

describe("describeLendingError", () => {
  it("maps known LendingError codes", () => {
    expect(describeLendingError("HostError: Error(Contract, #6)")).toBe("This wallet has no open loan.");
    expect(describeLendingError("Error(Contract, #9)")).toMatch(/token transfer failed/);
  });

  it("falls back for unknown codes and raw messages", () => {
    expect(describeLendingError("Error(Contract, #99)")).toBe(
      "Lending contract rejected the transaction (error #99)."
    );
    expect(describeLendingError("network down")).toBe("network down");
  });
});

describe("fetchLoanPosition", () => {
  it("decodes the LoanRecord returned by get_loan", async () => {
    simulateTransactionMock.mockResolvedValue({ result: { retval: loanRecordScVal() } });

    const loan = await fetchLoanPosition(BORROWER);

    expect(loan).toEqual({
      loanId: BigInt(7),
      borrower: BORROWER,
      asset: ASSET,
      principal: BigInt(9_000_0000000),
      collateralAmount: BigInt(15_000_0000000),
      collateralToken: COLLATERAL,
      borrowTime: 1_700_000_000,
      dueDate: 1_702_592_000,
      interestRateBps: 500,
    });

    const tx = simulateTransactionMock.mock.calls[0][0] as Transaction;
    const invoke = tx.operations[0] as unknown as { func: xdr.HostFunction };
    expect(invoke.func.invokeContract().functionName().toString()).toBe("get_loan");
  });

  it("returns null when the borrower has no loan (Option::None)", async () => {
    simulateTransactionMock.mockResolvedValue({ result: { retval: xdr.ScVal.scvVoid() } });
    await expect(fetchLoanPosition(BORROWER)).resolves.toBeNull();
  });

  it("surfaces simulation errors as friendly messages", async () => {
    simulateTransactionMock.mockResolvedValue({ error: "HostError: Error(Contract, #33)", events: [] });
    await expect(fetchLoanPosition(BORROWER)).rejects.toThrow("The lending contract is paused.");
  });

  it("requires the lending contract id", async () => {
    process.env.NEXT_PUBLIC_LENDING_CONTRACT_ID = "";
    await expect(fetchLoanPosition(BORROWER)).rejects.toBeInstanceOf(ContractSimulationError);
  });
});

describe("fetchRepaymentQuote", () => {
  it("splits the contract payoff into principal, interest and late fee", async () => {
    simulateTransactionMock.mockImplementation(async (tx: Transaction) => {
      const invoke = tx.operations[0] as unknown as { func: xdr.HostFunction };
      const fn = invoke.func.invokeContract().functionName().toString();
      const value = fn === "get_repayment_amount" ? BigInt(1_060) : BigInt(10);
      return { result: { retval: nativeToScVal(value, { type: "u64" }) } };
    });

    await expect(fetchRepaymentQuote(BORROWER, BigInt(1_000))).resolves.toEqual({
      principal: BigInt(1_000),
      interest: BigInt(50),
      lateFee: BigInt(10),
      total: BigInt(1_060),
    });
  });
});

describe("buildRepayLoanTransaction", () => {
  it("builds and prepares a repay(borrower) invocation", async () => {
    getAccountMock.mockResolvedValue(new Account(BORROWER, "42"));
    prepareTransactionMock.mockImplementation(async (tx: Transaction) => tx);

    const result = await buildRepayLoanTransaction(BORROWER);

    expect(result.networkPassphrase).toBe(STELLAR_NETWORK_PASSPHRASE);
    const tx = new Transaction(result.unsignedTransactionXdr, STELLAR_NETWORK_PASSPHRASE);
    const invoke = (tx.operations[0] as unknown as { func: xdr.HostFunction }).func.invokeContract();
    expect(invoke.functionName().toString()).toBe("repay");
    expect(Address.fromScVal(invoke.args()[0]).toString()).toBe(BORROWER);
    expect(Address.fromScAddress(invoke.contractAddress()).toString()).toBe(LENDING_ID);
  });

  it("maps simulation failures from prepareTransaction", async () => {
    getAccountMock.mockResolvedValue(new Account(BORROWER, "42"));
    prepareTransactionMock.mockRejectedValue(new Error("HostError: Error(Contract, #9)"));
    await expect(buildRepayLoanTransaction(BORROWER)).rejects.toThrow(/token transfer failed/);
  });

  it("explains unfunded accounts", async () => {
    getAccountMock.mockRejectedValue(new Error("not found"));
    await expect(buildRepayLoanTransaction(BORROWER)).rejects.toThrow(/funded on this network/);
  });
});
