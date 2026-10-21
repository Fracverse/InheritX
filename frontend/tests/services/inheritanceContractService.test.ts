import { describe, expect, it, vi, beforeEach } from "vitest";

const getAccountMock = vi.fn();
const prepareTransactionMock = vi.fn();

vi.mock("@stellar/stellar-sdk", async () => {
  const actual = await vi.importActual<typeof import("@stellar/stellar-sdk")>(
    "@stellar/stellar-sdk"
  );
  return {
    ...actual,
    rpc: {
      ...actual.rpc,
      Server: vi.fn().mockImplementation(() => ({
        getAccount: getAccountMock,
        prepareTransaction: prepareTransactionMock,
      })),
    },
  };
});

vi.mock("@/app/lib/api/client", async () => {
  const actual = await vi.importActual<typeof import("@/app/lib/api/client")>(
    "@/app/lib/api/client"
  );
  return {
    ...actual,
    apiClient: {
      post: vi.fn(),
    },
  };
});

import { Account } from "@stellar/stellar-sdk";
import { apiClient } from "@/app/lib/api/client";
import {
  invokeCreateInheritancePlan,
  submitSignedTransaction,
  TransactionSubmissionError,
  type CreateInheritancePlanContractInput,
} from "@/app/services/inheritanceContractService";

const baseInput: CreateInheritancePlanContractInput = {
  owner: "GAIE4IHLNGMX2ZGURV2DZEAFFU6P3X7UPFNRSGRZI2QUD2IU4GVOMKIV",
  token: "GDP2PVYRRAB35TQMJ4DPJOV5BBBM6PJUHWKMSTF6YYXUDXUT7BCLCIFE",
  planName: "Family Trust",
  description: "A test plan",
  amount: "100",
  distributionMethod: "LumpSum",
  isLendable: false,
  beneficiaries: [
    {
      fullName: "Alice",
      email: "alice@example.com",
      claimCode: 123456,
      bankAccount: "0123456789",
      allocationBp: 10000,
      priority: 1,
    },
  ],
};

describe("inheritanceContractService", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    process.env.NEXT_PUBLIC_INHERITANCE_CONTRACT_ID =
      "CAAQCAIBAEAQCAIBAEAQCAIBAEAQCAIBAEAQCAIBAEAQCAIBAEAQC526";

    getAccountMock.mockResolvedValue(
      new Account(baseInput.owner, "123")
    );
    prepareTransactionMock.mockImplementation(async (tx: { toXDR: () => string }) => tx);
  });

  it("builds, signs and submits the create_inheritance_plan transaction", async () => {
    const signTransaction = vi.fn().mockResolvedValue({ signedTxXdr: "signed-xdr" });
    vi.mocked(apiClient.post).mockResolvedValue({
      hash: "abc123",
      successful: true,
    });

    const progress: string[] = [];

    const result = await invokeCreateInheritancePlan({
      contractInput: baseInput,
      walletAddress: baseInput.owner,
      signTransaction,
      onProgress: (p) => progress.push(p.stage),
    });

    expect(getAccountMock).toHaveBeenCalledWith(baseInput.owner);
    expect(prepareTransactionMock).toHaveBeenCalled();
    expect(signTransaction).toHaveBeenCalledWith(
      expect.any(String),
      expect.objectContaining({ address: baseInput.owner })
    );
    expect(apiClient.post).toHaveBeenCalledWith("/api/transactions/submit", {
      xdr: "signed-xdr",
    });
    expect(result.submission.hash).toBe("abc123");
    expect(result.explorerUrl).toContain("abc123");
    expect(progress).toEqual(["building", "awaiting-signature", "submitting", "confirmed"]);
  });

  it("throws TransactionSubmissionError when the network returns no hash", async () => {
    vi.mocked(apiClient.post).mockResolvedValue({ successful: true });

    await expect(submitSignedTransaction("signed-xdr")).rejects.toThrow(
      TransactionSubmissionError
    );
  });

  it("dispatches transaction-confirmed only when Horizon marks the submission successful", async () => {
    const dispatchSpy = vi.spyOn(window, "dispatchEvent");
    vi.mocked(apiClient.post).mockResolvedValue({ hash: "pending-hash", successful: false });

    const result = await submitSignedTransaction("signed-xdr");

    expect(result.successful).toBe(false);
    expect(dispatchSpy).not.toHaveBeenCalledWith(
      expect.objectContaining({ type: "stellar:transaction-confirmed" })
    );

    dispatchSpy.mockClear();
    vi.mocked(apiClient.post).mockResolvedValue({ hash: "confirmed-hash", successful: true });
    await submitSignedTransaction("signed-xdr");

    expect(dispatchSpy).toHaveBeenCalledWith(
      expect.objectContaining({ type: "stellar:transaction-confirmed" })
    );
    dispatchSpy.mockRestore();
  });

  it("does not treat a response without an explicit success flag as confirmed", async () => {
    const dispatchSpy = vi.spyOn(window, "dispatchEvent");
    vi.mocked(apiClient.post).mockResolvedValue({ hash: "unknown-status" });

    const result = await submitSignedTransaction("signed-xdr");

    expect(result.successful).toBe(false);
    expect(dispatchSpy).not.toHaveBeenCalledWith(
      expect.objectContaining({ type: "stellar:transaction-confirmed" })
    );
    dispatchSpy.mockRestore();
  });
});
