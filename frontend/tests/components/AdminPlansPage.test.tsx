import { describe, it, expect, beforeEach, vi, afterEach, type MockInstance } from "vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { http, HttpResponse } from "msw";
import { server } from "../mocks/server";
import { apiClient } from "@/app/lib/api/client";
import AdminPlansPage from "@/app/admin/plans/page";

type PlanRecord = {
  id: string;
  owner_address: string;
  token_address: string;
  amount: string;
  yield_rate_bps: number;
  status: string;
  last_ping: number;
  heirs_count: number;
};

const basePlans = (): PlanRecord[] => [
  {
    id: "plan_1",
    owner_address: "GOWNER1",
    token_address: "GTOKEN",
    amount: "32000",
    yield_rate_bps: 450,
    status: "ACTIVE",
    last_ping: 1_752_487_200,
    heirs_count: 2,
  },
  {
    id: "plan_2",
    owner_address: "GOWNER2",
    token_address: "GTOKEN",
    amount: "50000",
    yield_rate_bps: 500,
    status: "COMPLETED",
    last_ping: 1_752_400_000,
    heirs_count: 3,
  },
];

const listResponse = (data: PlanRecord[]) => ({
  status: "success",
  data,
  page: 1,
  limit: 100,
  total_count: data.length,
  total_pages: 1,
  has_next: false,
  has_prev: false,
});

function servePlans(plans: PlanRecord[] = basePlans()) {
  server.use(http.get("*/api/admin/plans", () => HttpResponse.json(listResponse(plans))));
}

const rowFor = (planId: string) => screen.getByText(planId).closest("tr")!;
const terminateButton = (planId: string) =>
  within(rowFor(planId)).getByTitle("Force Complete / Terminate Plan");

let confirmSpy: MockInstance<typeof window.confirm>;

beforeEach(() => {
  localStorage.setItem("auth_token", "admin-jwt");
  apiClient.setDefaultRetry({ maxRetries: 0 });
  confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(true);
});

afterEach(() => {
  confirmSpy.mockRestore();
});

describe("AdminPlansPage - initial loading", () => {
  it("shows a loading state, then the plans returned by the server", async () => {
    let release!: () => void;
    const gate = new Promise<void>((r) => (release = r));
    server.use(
      http.get("*/api/admin/plans", async () => {
        await gate;
        return HttpResponse.json(listResponse(basePlans()));
      })
    );

    render(<AdminPlansPage />);
    expect(screen.getByRole("status")).toHaveTextContent("Loading plans");
    expect(screen.queryByText("No matching plans found.")).not.toBeInTheDocument();

    release();
    expect(await screen.findByText("plan_1")).toBeInTheDocument();
    expect(screen.getByText("plan_2")).toBeInTheDocument();
    expect(within(rowFor("plan_1")).getByText("ACTIVE")).toBeInTheDocument();
    expect(within(rowFor("plan_1")).getByText("4.50%")).toBeInTheDocument();
    expect(screen.queryByRole("status")).not.toBeInTheDocument();
  });

  it("requests plans with the admin bearer token and no longer shows mock plans", async () => {
    let auth: string | null = null;
    server.use(
      http.get("*/api/admin/plans", ({ request }) => {
        auth = request.headers.get("authorization");
        return HttpResponse.json(listResponse(basePlans()));
      })
    );

    render(<AdminPlansPage />);
    await screen.findByText("plan_1");

    expect(auth).toBe("Bearer admin-jwt");
    expect(screen.queryByText("plan_01jm8v45px")).not.toBeInTheDocument(); // old mock plan
  });
});

describe("AdminPlansPage - persisted actions", () => {
  it("terminates a plan via the API and shows the status the server returned", async () => {
    const user = userEvent.setup();
    servePlans();
    server.use(
      http.post("*/api/admin/plans/plan_1/terminate", () =>
        HttpResponse.json({ ...basePlans()[0], status: "COMPLETED" })
      )
    );

    render(<AdminPlansPage />);
    await screen.findByText("plan_1");
    expect(terminateButton("plan_1")).toBeEnabled();

    await user.click(terminateButton("plan_1"));

    await waitFor(() =>
      expect(within(rowFor("plan_1")).getByText("COMPLETED")).toBeInTheDocument()
    );
    expect(terminateButton("plan_1")).toBeDisabled();
    expect(confirmSpy).toHaveBeenCalledWith(expect.stringContaining("plan_1"));
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("sends no request when the admin cancels the confirmation", async () => {
    const user = userEvent.setup();
    confirmSpy.mockReturnValue(false);
    let called = false;
    servePlans();
    server.use(
      http.post("*/api/admin/plans/:id/terminate", () => {
        called = true;
        return HttpResponse.json(basePlans()[0]);
      })
    );

    render(<AdminPlansPage />);
    await screen.findByText("plan_1");
    await user.click(terminateButton("plan_1"));

    expect(called).toBe(false);
    expect(within(rowFor("plan_1")).getByText("ACTIVE")).toBeInTheDocument();
  });

  it("disables the button while the request is in flight, so it can't be double-submitted", async () => {
    const user = userEvent.setup();
    let release!: () => void;
    const gate = new Promise<void>((r) => (release = r));
    let requests = 0;
    servePlans();
    server.use(
      http.post("*/api/admin/plans/plan_1/terminate", async () => {
        requests += 1;
        await gate;
        return HttpResponse.json({ ...basePlans()[0], status: "COMPLETED" });
      })
    );

    render(<AdminPlansPage />);
    await screen.findByText("plan_1");
    await user.click(terminateButton("plan_1"));

    await waitFor(() => expect(terminateButton("plan_1")).toBeDisabled());
    // Status is not changed locally while waiting on the server.
    expect(within(rowFor("plan_1")).getByText("ACTIVE")).toBeInTheDocument();

    release();
    await waitFor(() =>
      expect(within(rowFor("plan_1")).getByText("COMPLETED")).toBeInTheDocument()
    );
    expect(requests).toBe(1);
  });

  it("survives a refresh: remounting loads the state the server persisted", async () => {
    const user = userEvent.setup();
    const db = basePlans(); // stands in for the server's database
    server.use(
      http.get("*/api/admin/plans", () => HttpResponse.json(listResponse(db))),
      http.post("*/api/admin/plans/:id/terminate", ({ params }) => {
        const record = db.find((p) => p.id === params.id)!;
        record.status = "COMPLETED";
        return HttpResponse.json(record);
      })
    );

    const first = render(<AdminPlansPage />);
    await screen.findByText("plan_1");
    await user.click(terminateButton("plan_1"));
    await waitFor(() => expect(terminateButton("plan_1")).toBeDisabled());
    first.unmount();

    render(<AdminPlansPage />);
    await screen.findByText("plan_1");
    expect(within(rowFor("plan_1")).getByText("COMPLETED")).toBeInTheDocument();
    expect(terminateButton("plan_1")).toBeDisabled();
  });
});

describe("AdminPlansPage - API errors", () => {
  it("shows an actionable error and no table when loading fails (401)", async () => {
    server.use(
      http.get("*/api/admin/plans", () =>
        HttpResponse.json({ error: "Unauthorized" }, { status: 401 })
      )
    );

    render(<AdminPlansPage />);

    expect(await screen.findByRole("alert")).toHaveTextContent(/Loading plans failed.*Sign in again/);
    expect(screen.queryByRole("table")).not.toBeInTheDocument();
    expect(screen.queryByText("No matching plans found.")).not.toBeInTheDocument();
  });

  it("recovers when the admin retries a failed load", async () => {
    const user = userEvent.setup();
    servePlans();
    // Registered last, so it takes priority for the first request only.
    server.use(
      http.get(
        "*/api/admin/plans",
        () => HttpResponse.json({ error: "Service unavailable" }, { status: 503 }),
        { once: true }
      )
    );

    render(<AdminPlansPage />);
    expect(await screen.findByRole("alert")).toHaveTextContent("Service unavailable");

    await user.click(screen.getByRole("button", { name: "Retry" }));

    expect(await screen.findByText("plan_1")).toBeInTheDocument();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("keeps the plan unchanged and shows the server's reason when termination is rejected (409)", async () => {
    const user = userEvent.setup();
    servePlans();
    server.use(
      http.post("*/api/admin/plans/plan_1/terminate", () =>
        HttpResponse.json({ error: "Plan is already inactive" }, { status: 409 })
      )
    );

    render(<AdminPlansPage />);
    await screen.findByText("plan_1");
    await user.click(terminateButton("plan_1"));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Terminating plan plan_1 failed: Plan is already inactive"
    );
    expect(within(rowFor("plan_1")).getByText("ACTIVE")).toBeInTheDocument();
    expect(terminateButton("plan_1")).toBeEnabled(); // can be retried
  });

  it("explains a permission failure (403)", async () => {
    const user = userEvent.setup();
    servePlans();
    server.use(
      http.post("*/api/admin/plans/plan_1/terminate", () =>
        HttpResponse.json({ error: "Forbidden" }, { status: 403 })
      )
    );

    render(<AdminPlansPage />);
    await screen.findByText("plan_1");
    await user.click(terminateButton("plan_1"));

    expect(await screen.findByRole("alert")).toHaveTextContent(/not permitted/);
    expect(within(rowFor("plan_1")).getByText("ACTIVE")).toBeInTheDocument();
  });

  it("reports a network failure without changing the plan", async () => {
    const user = userEvent.setup();
    servePlans();
    server.use(http.post("*/api/admin/plans/plan_1/terminate", () => HttpResponse.error()));

    render(<AdminPlansPage />);
    await screen.findByText("plan_1");
    await user.click(terminateButton("plan_1"));

    expect(await screen.findByRole("alert")).toHaveTextContent(/could not reach the server/);
    expect(within(rowFor("plan_1")).getByText("ACTIVE")).toBeInTheDocument();
    expect(terminateButton("plan_1")).toBeEnabled();
  });

  it("treats an empty 200 response as a failure, not a success", async () => {
    const user = userEvent.setup();
    servePlans();
    server.use(
      http.post("*/api/admin/plans/plan_1/terminate", () => new HttpResponse(null, { status: 200 }))
    );

    render(<AdminPlansPage />);
    await screen.findByText("plan_1");
    await user.click(terminateButton("plan_1"));

    expect(await screen.findByRole("alert")).toHaveTextContent(/unexpected response/);
    expect(within(rowFor("plan_1")).getByText("ACTIVE")).toBeInTheDocument();
  });

  it("lets the admin dismiss an action error", async () => {
    const user = userEvent.setup();
    servePlans();
    server.use(
      http.post("*/api/admin/plans/plan_1/terminate", () =>
        HttpResponse.json({ error: "nope" }, { status: 409 })
      )
    );

    render(<AdminPlansPage />);
    await screen.findByText("plan_1");
    await user.click(terminateButton("plan_1"));
    await screen.findByRole("alert");

    await user.click(screen.getByRole("button", { name: "Dismiss" }));
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });
});
