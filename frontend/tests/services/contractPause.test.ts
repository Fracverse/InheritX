import { describe, expect, it, vi, beforeEach } from "vitest";

const getAccountMock = vi.fn();
const prepareTransactionMock = vi.fn();
const simulateTransactionMock = vi.fn();

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
        simulateTransaction: simulateTransactionMock,
      })),
    },
  };
});

vi.mock("@/app/lib/api/client", async () => {
  const actual = await vi.importActual<typeof import("@/app/lib/api/client")>(
    "@/app/lib/api/client"
  );
  return { ...actual, apiClient: { post: vi.fn() } };
});

import { Account, TransactionBuilder, nativeToScVal } from "@stellar/stellar-sdk";
import { apiClient } from "@/app/lib/api/client";
import {
  ContractSimulationError,
  TransactionSubmissionError,
  invokeContractPause,
  readContractPaused,
} from "@/app/services/inheritanceContractService";
import { STELLAR_NETWORK_PASSPHRASE } from "@/app/lib/stellar/network";

const ADMIN = "GAIE4IHLNGMX2ZGURV2DZEAFFU6P3X7UPFNRSGRZI2QUD2IU4GVOMKIV";
const CONTRACT_ID = "CAAQCAIBAEAQCAIBAEAQCAIBAEAQCAIBAEAQCAIBAEAQCAIBAEAQC526";

/** Name of the single contract function invoked by an unsigned XDR. */
function invokedFunction(xdr: string): string {
  const tx = TransactionBuilder.fromXDR(xdr, STELLAR_NETWORK_PASSPHRASE) as unknown as {
    operations: { func: { invokeContract: () => { functionName: () => { toString: () => string } } } }[];
  };
  return tx.operations[0].func.invokeContract().functionName().toString();
}

describe("invokeContractPause", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    process.env.NEXT_PUBLIC_INHERITANCE_CONTRACT_ID = CONTRACT_ID;
    getAccountMock.mockResolvedValue(new Account(ADMIN, "123"));
    prepareTransactionMock.mockImplementation(async (tx: unknown) => tx);
    vi.mocked(apiClient.post).mockResolvedValue({ hash: "pause-hash", successful: true });
  });

  it.each(["pause", "unpause"] as const)(
    "builds %s(admin), has the admin wallet sign it, and submits it",
    async (action) => {
      const stages: string[] = [];
      const signTransaction = vi.fn().mockResolvedValue({ signedTxXdr: "SIGNED" });

      const result = await invokeContractPause({
        admin: ADMIN,
        action,
        signTransaction,
        onProgress: ({ stage }) => stages.push(stage),
      });

      const [unsignedXdr, opts] = signTransaction.mock.calls[0];
      expect(invokedFunction(unsignedXdr)).toBe(action);
      expect(opts).toEqual({ networkPassphrase: STELLAR_NETWORK_PASSPHRASE, address: ADMIN });
      expect(apiClient.post).toHaveBeenCalledWith("/api/transactions/submit", { xdr: "SIGNED" });
      expect(stages).toEqual(["building", "awaiting-signature", "submitting", "confirmed"]);
      expect(result.submission.hash).toBe("pause-hash");
      expect(result.explorerUrl).toContain("pause-hash");
    }
  );

  it("surfaces a non-admin rejection from simulation before asking for a signature", async () => {
    prepareTransactionMock.mockRejectedValue(new Error("HostError: Error(Contract, #22)"));
    const signTransaction = vi.fn();

    await expect(
      invokeContractPause({ admin: ADMIN, action: "pause", signTransaction })
    ).rejects.toThrow("not an admin");
    expect(signTransaction).not.toHaveBeenCalled();
    expect(apiClient.post).not.toHaveBeenCalled();
  });

  it("reports a declined signature and submits nothing", async () => {
    const signTransaction = vi.fn().mockRejectedValue(new Error("User declined access"));

    await expect(
      invokeContractPause({ admin: ADMIN, action: "pause", signTransaction })
    ).rejects.toThrow("Signature request was declined in the wallet.");
    expect(apiClient.post).not.toHaveBeenCalled();
  });

  it("treats an unsuccessful submission as a failure", async () => {
    vi.mocked(apiClient.post).mockResolvedValue({ hash: "failed-hash", successful: false });
    const signTransaction = vi.fn().mockResolvedValue({ signedTxXdr: "SIGNED" });

    await expect(
      invokeContractPause({ admin: ADMIN, action: "pause", signTransaction })
    ).rejects.toBeInstanceOf(TransactionSubmissionError);
  });
});

describe("readContractPaused", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    process.env.NEXT_PUBLIC_INHERITANCE_CONTRACT_ID = CONTRACT_ID;
    getAccountMock.mockResolvedValue(new Account(ADMIN, "123"));
  });

  it.each([true, false])("returns is_paused() = %s from simulation", async (paused) => {
    simulateTransactionMock.mockResolvedValue({
      result: { retval: nativeToScVal(paused) },
      transactionData: {},
      events: [],
      latestLedger: 1,
      _parsed: true,
    });

    await expect(readContractPaused(ADMIN)).resolves.toBe(paused);
  });

  it("throws when simulation fails", async () => {
    simulateTransactionMock.mockResolvedValue({ error: "boom", latestLedger: 1, _parsed: true });
    await expect(readContractPaused(ADMIN)).rejects.toBeInstanceOf(ContractSimulationError);
  });
});
