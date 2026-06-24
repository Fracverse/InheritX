import { render, screen, fireEvent } from "@testing-library/react";
import { vi, describe, it, expect, beforeEach } from "vitest";
import DashboardLayout from "../DashboardLayout";
import { useWallet } from "@/hooks/useWallet";
import { usePathname } from "next/navigation";

vi.mock("@/hooks/useWallet", () => ({
  useWallet: vi.fn(),
}));

vi.mock("next/navigation", () => ({
  usePathname: vi.fn(),
}));

describe("DashboardLayout — Premium Layout Verification", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (usePathname as any).mockReturnValue("/dashboard/create");
  });

  it("should mount correct links and dynamically render balance counters if verified connections exist", () => {
    (useWallet as any).mockReturnValue({
      connected: true,
      publicKey: "GB23...INX",
      balance: "750.25",
      assetSymbol: "XLM",
    });

    render(
      <DashboardLayout>
        <div data-testid="child-context">Core Grid Elements</div>
      </DashboardLayout>
    );

    expect(screen.getByText("750.25 XLM")).toBeInTheDocument();
    expect(screen.getByTestId("child-context")).toBeInTheDocument();
    
    // Core routes matching specs visible
    expect(screen.getByText("Create Plan")).toBeInTheDocument();
    expect(screen.getByText("KYC Verification")).toBeInTheDocument();
  });

  it("should support continuous connection invocation pathways if identity arrays are drops", () => {
    const mockConnect = vi.fn();
    (useWallet as any).mockReturnValue({
      connected: false,
      publicKey: null,
      connect: mockConnect,
    });

    render(<DashboardLayout>{null}</DashboardLayout>);

    const connectButton = screen.getByRole("button", { name: "Connect Wallet" });
    fireEvent.click(connectButton);
    expect(mockConnect).toHaveBeenCalledTimes(1);
  });
});