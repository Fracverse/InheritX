import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { http, HttpResponse } from "msw";
import { server } from "../mocks/server";
import { EditInheritancePlanPanel } from "@/components/plans/EditInheritancePlanPanel";
import type { Plan } from "@/app/lib/api/plans";
import { apiClient } from "@/app/lib/api/client";

vi.mock("@/context/WalletContext", () => ({
  useWallet: vi.fn().mockReturnValue({
    kit: null,
    selectedWalletId: null,
    isConnected: true,
    address: "GMOCK123",
  }),
}));

vi.mock("framer-motion", () => ({
  motion: {
    div: ({ children, className, onClick, ...rest }: any) => (
      <div className={className} onClick={onClick} {...rest}>
        {children}
      </div>
    ),
  },
  AnimatePresence: ({ children }: any) => <>{children}</>,
}));

const mockPlan: Plan = {
  id: "plan_1",
  user_id: "user_1",
  title: "Family Trust Plan",
  description: "A plan for the family",
  fee: 100,
  net_amount: 49900,
  status: "active",
  beneficiary_name: "Alice Smith",
  risk_override_enabled: false,
  created_at: "2024-01-01T00:00:00Z",
  updated_at: "2024-01-01T00:00:00Z",
};

beforeEach(() => {
  (apiClient as any).baseUrl = "";
});

describe("EditInheritancePlanPanel", () => {
  const mockOnClose = vi.fn();
  const mockOnSaved = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders with pre-populated plan data", () => {
    render(
      <EditInheritancePlanPanel
        plan={mockPlan}
        onClose={mockOnClose}
        onSaved={mockOnSaved}
      />
    );

    expect(screen.getByDisplayValue("Family Trust Plan")).toBeInTheDocument();
    expect(screen.getByDisplayValue("A plan for the family")).toBeInTheDocument();
  });

  it("shows the plan id in the header", () => {
    render(
      <EditInheritancePlanPanel
        plan={mockPlan}
        onClose={mockOnClose}
        onSaved={mockOnSaved}
      />
    );

    expect(screen.getByText(/ID: plan_1/)).toBeInTheDocument();
  });

  it("calls onClose when the close button is clicked", async () => {
    const user = userEvent.setup();
    render(
      <EditInheritancePlanPanel
        plan={mockPlan}
        onClose={mockOnClose}
        onSaved={mockOnSaved}
      />
    );

    await user.click(screen.getByLabelText("Close panel"));
    expect(mockOnClose).toHaveBeenCalledTimes(1);
  });

  it("calls onClose when Cancel button is clicked", async () => {
    const user = userEvent.setup();
    render(
      <EditInheritancePlanPanel
        plan={mockPlan}
        onClose={mockOnClose}
        onSaved={mockOnSaved}
      />
    );

    await user.click(screen.getByRole("button", { name: /cancel/i }));
    expect(mockOnClose).toHaveBeenCalledTimes(1);
  });

  it("pre-populates the beneficiary name from the plan", () => {
    render(
      <EditInheritancePlanPanel
        plan={mockPlan}
        onClose={mockOnClose}
        onSaved={mockOnSaved}
      />
    );

    expect(screen.getByDisplayValue("Alice Smith")).toBeInTheDocument();
  });

  it("adds a new beneficiary row when Add beneficiary is clicked", async () => {
    const user = userEvent.setup();
    render(
      <EditInheritancePlanPanel
        plan={mockPlan}
        onClose={mockOnClose}
        onSaved={mockOnSaved}
      />
    );

    const before = screen.getAllByPlaceholderText("G...");
    await user.click(screen.getByRole("button", { name: /add beneficiary/i }));
    const after = screen.getAllByPlaceholderText("G...");

    expect(after.length).toBe(before.length + 1);
  });

  it("removes a beneficiary row when delete is clicked and more than one exists", async () => {
    const user = userEvent.setup();
    render(
      <EditInheritancePlanPanel
        plan={mockPlan}
        onClose={mockOnClose}
        onSaved={mockOnSaved}
      />
    );

    // Add a second beneficiary so we can remove one
    await user.click(screen.getByRole("button", { name: /add beneficiary/i }));
    const before = screen.getAllByPlaceholderText("G...");

    const deleteButtons = screen.getAllByLabelText(/remove beneficiary/i);
    await user.click(deleteButtons[0]);
    const after = screen.getAllByPlaceholderText("G...");

    expect(after.length).toBe(before.length - 1);
  });

  it("shows allocation total as a badge", () => {
    render(
      <EditInheritancePlanPanel
        plan={mockPlan}
        onClose={mockOnClose}
        onSaved={mockOnSaved}
      />
    );

    // Default plan has one beneficiary at 100%
    expect(screen.getByText("100% / 100%")).toBeInTheDocument();
  });

  it("disables Save Changes when allocation is not 100%", async () => {
    const user = userEvent.setup();
    render(
      <EditInheritancePlanPanel
        plan={mockPlan}
        onClose={mockOnClose}
        onSaved={mockOnSaved}
      />
    );

    // Add a second beneficiary — total goes below 100
    await user.click(screen.getByRole("button", { name: /add beneficiary/i }));

    const saveButton = screen.getByRole("button", { name: /save changes/i });
    expect(saveButton).toBeDisabled();
  });

  it("renders the inactivity period field", () => {
    render(
      <EditInheritancePlanPanel
        plan={mockPlan}
        onClose={mockOnClose}
        onSaved={mockOnSaved}
      />
    );

    expect(screen.getByLabelText(/inactivity period/i)).toBeInTheDocument();
  });

  it("toggles yield harvesting on click", async () => {
    const user = userEvent.setup();
    render(
      <EditInheritancePlanPanel
        plan={mockPlan}
        onClose={mockOnClose}
        onSaved={mockOnSaved}
      />
    );

    const toggle = screen.getByRole("switch");
    expect(toggle).toHaveAttribute("aria-checked", "false");

    await user.click(toggle);
    expect(toggle).toHaveAttribute("aria-checked", "true");
  });

  it("submits plan update and calls onSaved on success", async () => {
    const user = userEvent.setup();
    render(
      <EditInheritancePlanPanel
        plan={mockPlan}
        onClose={mockOnClose}
        onSaved={mockOnSaved}
      />
    );

    await user.click(screen.getByRole("button", { name: /save changes/i }));

    await waitFor(
      () => {
        expect(mockOnSaved).toHaveBeenCalledTimes(1);
      },
      { timeout: 3000 }
    );
  });

  it("shows error message when API call fails", async () => {
    server.use(
      http.put("/api/plans/:id", () =>
        HttpResponse.json({ error: "Server error" }, { status: 500 })
      )
    );

    const user = userEvent.setup();
    render(
      <EditInheritancePlanPanel
        plan={mockPlan}
        onClose={mockOnClose}
        onSaved={mockOnSaved}
      />
    );

    await user.click(screen.getByRole("button", { name: /save changes/i }));

    await waitFor(
      () => {
        expect(
          screen.getByText(/server error|failed to save/i)
        ).toBeInTheDocument();
      },
      { timeout: 8000 }
    );
  });

  it("shows validation error when a beneficiary name is missing", async () => {
    const user = userEvent.setup();
    const planWithoutName: Plan = {
      ...mockPlan,
      beneficiary_name: undefined,
    };

    render(
      <EditInheritancePlanPanel
        plan={planWithoutName}
        onClose={mockOnClose}
        onSaved={mockOnSaved}
      />
    );

    // Find the Share (%) input in the Beneficiaries section and set to 100
    const shareInputs = screen.getAllByRole("spinbutton");
    const shareInput = shareInputs[0];
    if (shareInput) {
      await user.clear(shareInput);
      await user.type(shareInput, "100");
    }

    await user.click(screen.getByRole("button", { name: /save changes/i }));

    await waitFor(() => {
      expect(
        screen.getByText(/all beneficiaries must have a name and wallet address/i)
      ).toBeInTheDocument();
    });
  });
});
