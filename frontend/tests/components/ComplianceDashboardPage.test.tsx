import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import ComplianceDashboardPage from "@/app/admin/compliance/page";

// Mock window.URL.createObjectURL and revokeObjectURL
window.URL.createObjectURL = vi.fn().mockReturnValue("blob:http://localhost/mock-blob");
window.URL.revokeObjectURL = vi.fn();

// Mock global.window.setInterval and clearInterval
vi.spyOn(window, "setInterval").mockImplementation((fn: any) => {
  return 999 as any;
});
vi.spyOn(window, "clearInterval").mockImplementation(() => {});

// Mock framer-motion to avoid animation issues in tests
vi.mock("framer-motion", () => ({
  motion: {
    div: ({ children, className, initial, animate, exit, transition, ...props }: any) => (
      <div className={className} {...props}>
        {children}
      </div>
    ),
  },
  AnimatePresence: ({ children }: any) => <>{children}</>,
}));

describe("ComplianceDashboardPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("loads and displays velocity and volume alerts on mount", async () => {
    render(<ComplianceDashboardPage />);

    // Check loading text first
    expect(screen.getByText("Loading compliance alerts...")).toBeInTheDocument();

    // Wait for the alerts to render
    await waitFor(() => {
      expect(screen.getByText("High transaction velocity: 12 transfers in 10 minutes")).toBeInTheDocument();
      expect(screen.getByText("Large transfer volume: 150,000 USDC exceeds threshold of 100,000")).toBeInTheDocument();
    });

    // Check metric totals
    expect(screen.getByText("Velocity Alerts").closest("div")?.querySelector("p.text-2xl")?.textContent).toBe("1");
    expect(screen.getByText("Volume Alerts").closest("div")?.querySelector("p.text-2xl")?.textContent).toBe("1");
  });

  it("handles risk and sanctions screening for high risk address", async () => {
    render(<ComplianceDashboardPage />);

    await waitFor(() => {
      expect(screen.queryByText("Loading compliance alerts...")).not.toBeInTheDocument();
    });

    // Enter a high risk address (must contain RISK as mocked in MSW)
    const addressInput = screen.getByPlaceholderText("Enter wallet address");
    fireEvent.change(addressInput, { target: { value: "GDRISK7W7YQF4LQRYR6D2AH6FZKBX6E5D3EXAMPLEADDRESS" } });
    fireEvent.submit(addressInput.closest("form")!);

    // Wait for risk score and sanctions to render
    await waitFor(() => {
      expect(screen.getAllByText("85/100")[0]).toBeInTheDocument();
      expect(screen.getByText("Sanctions Association")).toBeInTheDocument();
      expect(screen.getByText("Address flagged")).toBeInTheDocument();
      expect(screen.getByText("OFAC SDN List")).toBeInTheDocument();
    });
  });

  it("handles risk and sanctions screening for low risk address", async () => {
    render(<ComplianceDashboardPage />);

    await waitFor(() => {
      expect(screen.queryByText("Loading compliance alerts...")).not.toBeInTheDocument();
    });

    // Enter a low risk address
    const addressInput = screen.getByPlaceholderText("Enter wallet address");
    fireEvent.change(addressInput, { target: { value: "GSAFE7W7YQF4LQRYR6D2AH6FZKBX6E5D3EXAMPLEADDRESS" } });
    fireEvent.submit(addressInput.closest("form")!);

    // Wait for risk score and sanctions to render
    await waitFor(() => {
      expect(screen.getAllByText("15/100")[0]).toBeInTheDocument();
      expect(screen.getByText("No sanctions match")).toBeInTheDocument();
    });
  });

  it("submits a risk override with justification", async () => {
    render(<ComplianceDashboardPage />);

    await waitFor(() => {
      expect(screen.queryByText("Loading compliance alerts...")).not.toBeInTheDocument();
    });

    // We can screen an address first to populate the override fields
    const addressInput = screen.getByPlaceholderText("Enter wallet address");
    fireEvent.change(addressInput, { target: { value: "GDRISK7W7YQF4LQRYR6D2AH6FZKBX6E5D3EXAMPLEADDRESS" } });
    fireEvent.submit(addressInput.closest("form")!);

    await waitFor(() => {
      expect(screen.getAllByText("85/100")[0]).toBeInTheDocument();
    });

    // Fill in justification
    const justificationTextarea = screen.getByPlaceholderText("Justification for audit trail");
    fireEvent.change(justificationTextarea, { target: { value: "Legitimate institutional wallet under compliance team review" } });

    // Click submit override
    fireEvent.submit(justificationTextarea.closest("form")!);

    // Verify success message and updated risk level badge (if overridden to level low/medium)
    await waitFor(() => {
      expect(screen.getByText("Risk override recorded for audit review.")).toBeInTheDocument();
    });
  });

  it("updates individual alert status via dropdown", async () => {
    render(<ComplianceDashboardPage />);

    await waitFor(() => {
      expect(screen.queryByText("Loading compliance alerts...")).not.toBeInTheDocument();
    });

    // Find a status select dropdown
    const selects = screen.getAllByRole("combobox");
    expect(selects.length).toBeGreaterThan(0);

    // Change value
    fireEvent.change(selects[0], { target: { value: "resolved" } });
    expect(selects[0]).toHaveValue("resolved");
  });

  it("supports report exports to CSV and JSON", async () => {
    const user = userEvent.setup();
    render(<ComplianceDashboardPage />);

    await waitFor(() => {
      expect(screen.queryByText("Loading compliance alerts...")).not.toBeInTheDocument();
    });

    const csvButton = screen.getByRole("button", { name: /csv/i });
    const jsonButton = screen.getByRole("button", { name: /json/i });

    const anchorClickSpy = vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(() => {});

    await user.click(csvButton);
    expect(window.URL.createObjectURL).toHaveBeenCalled();
    expect(anchorClickSpy).toHaveBeenCalled();

    await user.click(jsonButton);
    expect(window.URL.createObjectURL).toHaveBeenCalled();
    expect(anchorClickSpy).toHaveBeenCalled();
  });
});
