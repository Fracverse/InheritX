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
      expect(docs).toHaveLength(1);
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
