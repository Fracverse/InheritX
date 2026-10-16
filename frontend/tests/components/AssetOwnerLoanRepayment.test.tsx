import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { LoanPosition } from "@/app/lib/stellar/lendingClient";

vi.mock("next/link", () => ({
  default: ({ children, href, ...rest }: { children: React.ReactNode; href: string }) => (
    <a href={href} {...rest}>
      {children}
    </a>
  ),
}));

vi.mock("@/context/WalletContext", () => ({
  useWallet: () => ({ isConnected: true, address: "GBORROWER", signTransaction: vi.fn() }),
}));

vi.mock("@/lib/api/dataSource", () => ({
  useMockData: false,
  getPlans: vi.fn().mockResolvedValue([]),
}));

vi.mock("@/components/plans/InactivityTimerCard", () => ({ default: () => null }));

const UNIT = BigInt(10_000_000);
const riskyLoan: LoanPosition = {
  loanId: BigInt(9),
  borrower: "GBORROWER",
  asset: "USDC",
  principal: BigInt(7_000) * UNIT,
  collateralAmount: BigInt(10_000) * UNIT,
  collateralToken: "XLM",
  borrowTime: 1_700_000_000,
  dueDate: 4_102_444_800, // far future: not overdue
  interestRateBps: 800,
};
const quote = { principal: riskyLoan.principal, interest: BigInt(50) * UNIT, lateFee: BigInt(0), total: BigInt(7_050) * UNIT };

const getActiveLoan = vi.fn();
vi.mock("@/app/services/loanRepaymentService", () => ({
  getActiveLoan: (...args: unknown[]) => getActiveLoan(...args),
  getRepaymentQuote: vi.fn(async () => quote),
  invokeRepayLoan: vi.fn(async ({ onProgress }: { onProgress: (s: string) => void }) => {
    onProgress("confirmed");
    return { hash: "h", explorerUrl: null, quote };
  }),
}));

import AssetOwnerPage from "@/app/asset-owner/page";

describe("Asset owner dashboard — loan liquidation warning & repayment", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    getActiveLoan.mockResolvedValue(riskyLoan);
  });

  it("warns about liquidation and drops the LTV gauge to 0% after repayment", async () => {
    render(<AssetOwnerPage />);

    const card = (await screen.findByRole("heading", { name: /Loan Health/ })).closest("section")!;
    expect(within(card).getByTestId("liquidation-warning")).toHaveTextContent("Liquidation risk");
    expect(within(card).getByRole("meter")).toHaveAttribute("aria-valuenow", "70");
    expect(getActiveLoan).toHaveBeenCalledWith("GBORROWER");

    await userEvent.click(within(card).getByRole("button", { name: "Repay loan" }));
    const dialog = await screen.findByRole("dialog");
    await userEvent.click(await within(dialog).findByRole("button", { name: /Repay 7,050 USDC/ }));

    expect(await within(dialog).findByRole("heading", { name: "Loan repaid" })).toBeInTheDocument();

    // Dashboard gauge updated without a reload
    const updatedCard = screen.getByRole("heading", { name: /Loan Health/ }).closest("section")!;
    expect(within(updatedCard).getByRole("meter")).toHaveAttribute("aria-valuenow", "0");
    expect(within(updatedCard).queryByTestId("liquidation-warning")).not.toBeInTheDocument();
    expect(within(updatedCard).getByText(/Loan fully repaid/)).toBeInTheDocument();

    await userEvent.click(within(dialog).getByRole("button", { name: "Done" }));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("hides the loan section when the wallet has no loan", async () => {
    getActiveLoan.mockResolvedValue(null);
    render(<AssetOwnerPage />);
    await screen.findByRole("heading", { name: "Overview" });
    await vi.waitFor(() => expect(getActiveLoan).toHaveBeenCalled());
    expect(screen.queryByRole("heading", { name: /Loan Health/ })).not.toBeInTheDocument();
  });
});
