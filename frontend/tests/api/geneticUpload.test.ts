import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { webcrypto } from "node:crypto";
import {
  GeneticVerificationAPI,
  MAX_GENETIC_FILE_SIZE,
  validateGeneticFile,
} from "@/app/lib/api/geneticVerification";

const networkPassphrase = "Test SDF Network ; September 2015";
const api = new GeneticVerificationAPI("https://api.example.test");
function geneticFile(name = "sample.vcf") {
  const file = new File(["synthetic genetic fixture"], name);
  Object.defineProperty(file, "arrayBuffer", {
    value: async () =>
      new TextEncoder().encode("synthetic genetic fixture").buffer,
  });
  return file;
}

beforeEach(() => {
  vi.stubGlobal("crypto", webcrypto);
});
afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

describe("genetic file validation", () => {
  it.each(["sample.vcf", "sample.23andme", "sample.fasta", "SAMPLE.VCF"])(
    "accepts %s",
    (name) => {
      expect(() => validateGeneticFile(geneticFile(name))).not.toThrow();
    },
  );
  it.each(["sample.txt", "sample.vcf.exe", "vcf"])("rejects %s", (name) => {
    expect(() => validateGeneticFile(geneticFile(name))).toThrow("Choose a");
  });
  it("accepts exactly 25 MB and rejects one byte more", () => {
    const file = geneticFile();
    Object.defineProperty(file, "size", {
      value: MAX_GENETIC_FILE_SIZE,
      configurable: true,
    });
    expect(() => validateGeneticFile(file)).not.toThrow();
    Object.defineProperty(file, "size", { value: MAX_GENETIC_FILE_SIZE + 1 });
    expect(() => validateGeneticFile(file)).toThrow("25 MB");
  });
  it("rejects empty files", () => {
    expect(() => validateGeneticFile(new File([], "empty.vcf"))).toThrow(
      "empty",
    );
  });
});

describe("signed genetic proof submission", () => {
  it("hashes locally, signs the bound payload, and sends no file bytes or name", async () => {
    const signPayload = vi.fn().mockResolvedValue("wallet-signature");
    const stages: string[] = [];
    const fetch = vi.fn().mockImplementation(async (_url, init) => {
      const body = JSON.parse(init.body);
      return Response.json({
        data: { dnaHash: body.dna_hash, status: "pending" },
      });
    });
    vi.stubGlobal("fetch", fetch);
    const result = await api.uploadGeneticData("plan/1", geneticFile(), {
      address: "owner",
      networkPassphrase,
      signPayload,
      onStage: (stage) => stages.push(stage),
    });
    expect(stages).toEqual(["hashing", "signing", "uploading"]);
    expect(fetch).toHaveBeenCalledTimes(1);
    expect(fetch.mock.calls[0][0]).toBe(
      "https://api.example.test/api/genetic/plan%2F1/upload",
    );
    const body = JSON.parse(fetch.mock.calls[0][1].body);
    expect(Object.keys(body).sort()).toEqual([
      "dna_hash",
      "payload",
      "signature",
    ]);
    expect(body.payload).toBe(signPayload.mock.calls[0][0]);
    expect(JSON.parse(body.payload)).toMatchObject({
      purpose: "inheritx:genetic-upload:v1",
      plan_id: "plan/1",
      wallet_address: "owner",
      network_passphrase: networkPassphrase,
      dna_hash: result.dnaHash,
    });
    expect(result.dnaHash).toMatch(/^[a-f0-9]{64}$/);
    expect(JSON.stringify(body)).not.toContain("synthetic genetic fixture");
    expect(JSON.stringify(body)).not.toContain("sample.vcf");
  });
  it("does not sign or send invalid files or requests for another network", async () => {
    const signPayload = vi.fn();
    const fetch = vi.fn();
    vi.stubGlobal("fetch", fetch);
    await expect(
      api.uploadGeneticData("p", geneticFile("bad.exe"), {
        address: "owner",
        networkPassphrase,
        signPayload,
      }),
    ).rejects.toThrow("Choose a");
    await expect(
      api.uploadGeneticData("p", geneticFile(), {
        address: "owner",
        networkPassphrase: "mainnet",
        signPayload,
      }),
    ).rejects.toThrow("Testnet only");
    expect(signPayload).not.toHaveBeenCalled();
    expect(fetch).not.toHaveBeenCalled();
  });
  it("does not submit when the user rejects signing", async () => {
    const fetch = vi.fn();
    vi.stubGlobal("fetch", fetch);
    await expect(
      api.uploadGeneticData("p", geneticFile(), {
        address: "owner",
        networkPassphrase,
        signPayload: vi.fn().mockRejectedValue(new Error("Signature declined")),
      }),
    ).rejects.toThrow("Signature declined");
    expect(fetch).not.toHaveBeenCalled();
  });
  it("does not submit if cancelled while the wallet is signing", async () => {
    const controller = new AbortController();
    const fetch = vi.fn();
    vi.stubGlobal("fetch", fetch);
    await expect(
      api.uploadGeneticData("p", geneticFile(), {
        address: "owner",
        networkPassphrase,
        signal: controller.signal,
        signPayload: async () => {
          controller.abort();
          return "signature";
        },
      }),
    ).rejects.toThrow();
    expect(fetch).not.toHaveBeenCalled();
  });
  it.each([404, 501, 500])(
    "reports HTTP %s without retrying",
    async (status) => {
      const fetch = vi.fn().mockResolvedValue(new Response("", { status }));
      vi.stubGlobal("fetch", fetch);
      await expect(
        api.uploadGeneticData("p", geneticFile(), {
          address: "owner",
          networkPassphrase,
          signPayload: async () => "sig",
        }),
      ).rejects.toThrow(status === 500 ? "failed (500)" : "not available yet");
      expect(fetch).toHaveBeenCalledTimes(1);
    },
  );
  it("rejects a success response for the wrong hash", async () => {
    vi.stubGlobal(
      "fetch",
      vi
        .fn()
        .mockResolvedValue(
          Response.json({ data: { dnaHash: "wrong", status: "verified" } }),
        ),
    );
    await expect(
      api.uploadGeneticData("p", geneticFile(), {
        address: "owner",
        networkPassphrase,
        signPayload: async () => "sig",
      }),
    ).rejects.toThrow("invalid genetic");
  });
});
