import { describe, it, expect, beforeEach } from "vitest";
import { http, HttpResponse } from "msw";
import { server } from "../mocks/server";
import { MessagesAPI } from "@/app/lib/api/messages";

const getToken = () => "test-token";
let api: MessagesAPI;

beforeEach(() => {
  api = new MessagesAPI("", getToken);
});

describe("MessagesAPI", () => {
  describe("createMessage", () => {
    it("creates a new message", async () => {
      const msg = await api.createMessage({
        vault_id: "vault_1",
        title: "My Will Message",
        content: "Dear family...",
        unlock_at: "2030-01-01T00:00:00Z",
        beneficiary_ids: ["ben_1"],
      });
      expect(msg.id).toBe("msg_1");
      expect(msg.status).toBe("DRAFT");
      expect(msg.title).toBe("My Will Message");
    });

    it("throws on validation error", async () => {
      server.use(
        http.post("/api/messages/create", () =>
          HttpResponse.json({ error: "Title is required" }, { status: 422 }),
        ),
      );
      await expect(
        api.createMessage({
          vault_id: "vault_1",
          title: "",
          content: "",
          unlock_at: "",
          beneficiary_ids: [],
        }),
      ).rejects.toThrow("Title is required");
    });
  });

  describe("getMessage", () => {
    it("retrieves a message by id", async () => {
      const msg = await api.getMessage("msg_1");
      expect(msg.id).toBe("msg_1");
      expect(msg.status).toBe("DRAFT");
    });

    it("throws when message not found", async () => {
      server.use(
        http.get("/api/messages/:id", () =>
          HttpResponse.json({ error: "Not found" }, { status: 404 }),
        ),
      );
      await expect(api.getMessage("nonexistent")).rejects.toThrow("Not found");
    });
  });

  describe("updateMessage", () => {
    it("updates message fields", async () => {
      const updated = await api.updateMessage("msg_1", {
        title: "Updated Title",
      });
      expect(updated).toBeDefined();
    });
  });

  describe("finalizeMessage", () => {
    it("finalizes a message", async () => {
      const result = await api.finalizeMessage("msg_1");
      expect(result.success).toBe(true);
    });

    it("throws when message already finalized", async () => {
      server.use(
        http.post("/api/messages/:id/finalize", () =>
          HttpResponse.json(
            { error: "Message already finalized" },
            { status: 409 },
          ),
        ),
      );
      await expect(api.finalizeMessage("msg_1")).rejects.toThrow(
        "Message already finalized",
      );
    });
  });

  describe("deleteMessage", () => {
    it("deletes a message", async () => {
      const result = await api.deleteMessage("msg_1");
      expect(result.success).toBe(true);
    });
  });

  describe("getVaultMessages", () => {
    it("returns messages for a vault", async () => {
      const messages = await api.getVaultMessages("vault_1");
      expect(Array.isArray(messages)).toBe(true);
    });
  });

  describe("unlockMessage", () => {
    it("unlocks and returns decrypted content", async () => {
      const result = await api.unlockMessage("msg_1");
      expect(result.content).toBe("decrypted message content");
    });

    it("throws when unlock conditions not met", async () => {
      server.use(
        http.post("/api/messages/:id/unlock", () =>
          HttpResponse.json(
            { error: "Unlock conditions not met" },
            { status: 403 },
          ),
        ),
      );
      await expect(api.unlockMessage("msg_1")).rejects.toThrow(
        "Unlock conditions not met",
      );
    });
  });

  describe("getAccessAudit", () => {
    it("returns access audit logs", async () => {
      const logs = await api.getAccessAudit("msg_1");
      expect(Array.isArray(logs)).toBe(true);
    });
  });
});
