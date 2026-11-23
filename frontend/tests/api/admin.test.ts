import { describe, it, expect, beforeEach } from "vitest";
import { http, HttpResponse } from "msw";
import { server } from "../mocks/server";
import { AdminAPI } from "@/app/lib/api/admin";
import { apiClient } from "@/app/lib/api/client";

let api: AdminAPI;

beforeEach(() => {
  api = new AdminAPI();
  (apiClient as any).baseUrl = "";
});

describe("AdminAPI - plan listing", () => {
  it("calls the canonical /api/plans/due-for-claim route", async () => {
    // The previously client-only path must not be used.
    server.use(
      http.get("/api/admin/plans/due-for-claim", () =>
        HttpResponse.json({ error: "Unsupported path" }, { status: 404 }),
      ),
      http.get("/api/plans/due-for-claim", () =>
        HttpResponse.json({
          status: "ok",
          data: [{ id: "plan_1", status: "CLAIMABLE" }],
          page: 1,
          limit: 20,
          total_count: 1,
          total_pages: 1,
          has_next: false,
          has_prev: false,
        }),
      ),
    );

    const response = await api.getAllPlans(1, 20);
    expect(response.status).toBe("ok");
    expect(response.data).toHaveLength(1);
    expect(response.data[0].id).toBe("plan_1");
  });
});
