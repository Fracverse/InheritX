import { describe, it, expect, beforeEach } from "vitest";
import { http, HttpResponse } from "msw";
import { server } from "../mocks/server";
import { EmergencyAPI } from "@/app/lib/api/emergency";

const getToken = () => "test-token";
let api: EmergencyAPI;

beforeEach(() => {
  api = new EmergencyAPI("", getToken);
});

describe("EmergencyAPI", () => {
  describe("activateEmergency", () => {
    it("activates emergency access", async () => {
      const result = await api.activateEmergency("plan_1");
      expect(result.status).toBe("activated");
    });

    it("throws on unauthorized", async () => {
      server.use(
        http.post("/api/emergency/activate", () =>
          HttpResponse.json({ error: "Unauthorized" }, { status: 401 }),
        ),
      );
      await expect(api.activateEmergency("plan_1")).rejects.toThrow(
        "Unauthorized",
      );
    });
  });

  describe("addContact", () => {
    it("creates a new emergency contact", async () => {
      const contact = await api.addContact("plan_1", {
        name: "Alice",
        email: "alice@example.com",
        wallet_address: "GXYZ123",
      });
      expect(contact.id).toBe("contact_1");
      expect(contact.name).toBe("Alice");
    });
  });

  describe("removeContact", () => {
    it("removes a contact successfully", async () => {
      const result = await api.removeContact("contact_1");
      expect(result.success).toBe(true);
    });

    it("throws when contact not found", async () => {
      server.use(
        http.delete("/api/emergency/contacts/:id", () =>
          HttpResponse.json({ error: "Contact not found" }, { status: 404 }),
        ),
      );
      await expect(api.removeContact("nonexistent")).rejects.toThrow(
        "Contact not found",
      );
    });
  });

  describe("listContacts", () => {
    it("returns list of contacts", async () => {
      const contacts = await api.listContacts("plan_1");
      expect(contacts).toHaveLength(1);
      expect(contacts[0].name).toBe("Alice");
    });
  });

  describe("setGuardians", () => {
    it("sets guardians and threshold", async () => {
      const result = await api.setGuardians("plan_1", ["GXYZ123"], 1);
      expect(result.success).toBe(true);
    });
  });

  describe("approveRequest", () => {
    it("approves an emergency request", async () => {
      const result = await api.approveRequest("req_1");
      expect(result.success).toBe(true);
    });
  });

  describe("revokeAccess", () => {
    it("revokes emergency access", async () => {
      const result = await api.revokeAccess("plan_1");
      expect(result.success).toBe(true);
    });
  });

  describe("getAuditLogs", () => {
    it("returns audit logs", async () => {
      const logs = await api.getAuditLogs("plan_1");
      expect(logs).toHaveLength(1);
      expect(logs[0].action).toBe("ACTIVATE");
    });
  });
});
