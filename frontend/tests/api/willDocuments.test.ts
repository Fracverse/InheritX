import { describe, it, expect, beforeEach } from "vitest";
import { http, HttpResponse } from "msw";
import { server } from "../mocks/server";
import { WillDocumentsAPI } from "@/app/lib/api/willDocuments";

const getToken = () => "test-token";
let api: WillDocumentsAPI;

beforeEach(() => {
  api = new WillDocumentsAPI("", getToken);
});

describe("WillDocumentsAPI", () => {
  describe("listDocuments", () => {
    it("returns documents for a plan", async () => {
      const docs = await api.listDocuments("plan_1");
      expect(docs).toHaveLength(2);
      expect(docs[0].document_id).toBe("doc_1");
      expect(docs[0].version).toBe(1);
    });

    it("throws on auth error", async () => {
      server.use(
        http.get("/api/plans/:planId/will/documents", () =>
          HttpResponse.json({ error: "Unauthorized" }, { status: 401 }),
        ),
      );
      await expect(api.listDocuments("plan_1")).rejects.toThrow("Unauthorized");
    });
  });

  describe("getDocument", () => {
    it("retrieves a specific document", async () => {
      const doc = await api.getDocument("doc_1");
      expect(doc.document_id).toBe("doc_1");
      expect(doc.will_hash).toBe("abc123");
    });
  });

  describe("verifyDocument", () => {
    it("verifies document integrity", async () => {
      const result = await api.verifyDocument("doc_1");
      expect(result.is_valid).toBe(true);
      expect(result.hash_match).toBe(true);
    });

    it("returns invalid for tampered document", async () => {
      server.use(
        http.get("/api/will/documents/:documentId/verify", () =>
          HttpResponse.json({
            status: "ok",
            data: {
              is_valid: false,
              document_id: "doc_1",
              hash_match: false,
              message: "Hash mismatch detected",
            },
          }),
        ),
      );
      const result = await api.verifyDocument("doc_1");
      expect(result.is_valid).toBe(false);
      expect(result.hash_match).toBe(false);
    });
  });

  describe("generateDocument", () => {
    it("generates a new will document", async () => {
      const doc = await api.generateDocument("plan_1", {
        owner_name: "John Doe",
        owner_wallet: "GXYZ123",
        vault_id: "vault_1",
        beneficiaries: [
          {
            name: "Jane Doe",
            wallet_address: "GABC456",
            allocation_percent: "100",
            relationship: "spouse",
          },
        ],
      });
      expect(doc.document_id).toBe("doc_new");
      expect(doc.version).toBe(2);
    });
  });

  describe("uploadDocument", () => {
    it("encrypts the document before uploading", async () => {
      const requests: Record<string, unknown>[] = [];
      server.use(
        http.post("/api/plans/:planId/will/documents/upload", async ({ request }) => {
          const body = (await request.json()) as Record<string, unknown>;
          requests.push(body);
          return HttpResponse.json({
            status: "ok",
            data: {
              document_id: "doc_uploaded",
              plan_id: "plan_1",
              template_used: "application/pdf",
              will_hash: "encrypted-hash",
              generated_at: new Date().toISOString(),
              version: 3,
              filename: body.filename,
              is_encrypted: true,
            },
          }, { status: 201 });
        }),
      );

      const result = await api.uploadDocument(
        "plan_1",
        new Blob(["private will content"], { type: "application/pdf" }),
        async () => btoa("wallet-signature"),
        "will.pdf",
      );

      expect(result.is_encrypted).toBe(true);
      expect(requests[0]).toMatchObject({
        version: 1,
        algorithm: "AES-256-GCM",
        key_derivation: "HKDF-SHA-256",
        filename: "will.pdf",
      });
      expect(requests[0].ciphertext).not.toContain("private will content");
    });
  });

  describe("getPlanEvents", () => {
    it("returns events for a plan", async () => {
      const events = await api.getPlanEvents("plan_1");
      expect(Array.isArray(events)).toBe(true);
    });
  });

  describe("getPlanEventStats", () => {
    it("returns event statistics", async () => {
      const stats = await api.getPlanEventStats("plan_1");
      expect(stats.plan_id).toBe("plan_1");
      expect(stats.total_events).toBe(2);
      expect(stats.will_created_count).toBe(1);
    });
  });

  describe("authentication", () => {
    it("throws when no auth token provided", async () => {
      const unauthApi = new WillDocumentsAPI("", () => null);
      await expect(unauthApi.listDocuments("plan_1")).rejects.toThrow(
        "Authentication required",
      );
    });
  });
});
