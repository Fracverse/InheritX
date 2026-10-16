import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

const signTransaction = vi.fn();
vi.mock("@/context/WalletContext", () => ({
  useWallet: () => ({ signTransaction, address: "GBORROWER", isConnected: true }),
}));

const getRepaymentQuote = vi.fn();
const invokeRepayLoan = vi.fn();
vi.mock("@/app/services/loanRepaymentService", () => ({
  getRepaymentQuote: (...args: unknown[]) => getRepaymentQuote(...args),
  invokeRepayLoan: (...args: unknown[]) => invokeRepayLoan(...args),
}));

import { RepayLoanModal } from "@/components/loans/RepayLoanModal";
import type { LoanPosition } from "@/app/lib/stellar/lendingClient";

const UNIT = BigInt(10_000_000);
const loan: LoanPosition = {
  loanId: BigInt(3),
  borrower: "GBORROWER",
  asset: "USDC",
  principal: BigInt(6_200) * UNIT,
  collateralAmount: BigInt(10_000) * UNIT,
  collateralToken: "XLM",
  borrowTime: 1_700_000_000,
  dueDate: 1_702_592_000,
  interestRateBps: 800,
};
const quote = {
  principal: loan.principal,
  interest: BigInt(61) * UNIT,
  lateFee: BigInt(0),
  total: BigInt(6_261) * UNIT,
};

describe("RepayLoanModal", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    getRepaymentQuote.mockResolvedValue(quote);
  });

  it("shows the principal + interest breakdown and current vs projected LTV", async () => {
    render(<RepayLoanModal isOpen loan={loan} onClose={vi.fn()} />);

    expect(await screen.findByTestId("repay-total")).toHaveTextContent("6,261 USDC");
    expect(screen.getByText("6,200 USDC")).toBeInTheDocument();
    expect(screen.getByText("61 USDC")).toBeInTheDocument();
    expect(screen.getByRole("meter", { name: "Loan-to-value" })).toHaveAttribute("aria-valuenow", "62");
    expect(screen.getByTestId("ltv-gauge-projected")).toBeInTheDocument();
    expect(getRepaymentQuote).toHaveBeenCalledWith(loan);
  });

  it("repays via the Soroban flow, reports progress, and notifies the dashboard", async () => {
    const onRepaid = vi.fn();
    const result = { hash: "abc123", explorerUrl: "https://stellar.expert/explorer/testnet/tx/abc123", quote };
    invokeRepayLoan.mockImplementation(async ({ onProgress }) => {
      onProgress("awaiting-signature");
      return result;
    });

    render(<RepayLoanModal isOpen loan={loan} onClose={vi.fn()} onRepaid={onRepaid} />);
    await userEvent.click(await screen.findByRole("button", { name: /Repay 6,261 USDC/ }));

    expect(invokeRepayLoan).toHaveBeenCalledWith(
      expect.objectContaining({ loan, quote, signTransaction })
    );
    expect(await screen.findByRole("heading", { name: "Loan repaid" })).toBeInTheDocument();
    expect(onRepaid).toHaveBeenCalledWith(result);
    expect(screen.getByRole("meter", { name: "Loan-to-value after repayment" })).toHaveAttribute(
      "aria-valuenow",
      "0"
    );
    expect(screen.getByRole("link", { name: /Stellar Expert/ })).toHaveAttribute("href", result.explorerUrl);
  });

  it("shows contract/wallet errors and lets the user retry", async () => {
    invokeRepayLoan.mockRejectedValueOnce(new Error("Signature request was declined in the wallet."));
    const onRepaid = vi.fn();

    render(<RepayLoanModal isOpen loan={loan} onClose={vi.fn()} onRepaid={onRepaid} />);
    const repay = await screen.findByRole("button", { name: /Repay 6,261 USDC/ });
    await userEvent.click(repay);

    expect(await screen.findByText("Signature request was declined in the wallet.")).toBeInTheDocument();
    expect(onRepaid).not.toHaveBeenCalled();
    expect(repay).toBeEnabled();
  });

  it("disables repayment when the quote can't be loaded and offers a retry", async () => {
    getRepaymentQuote.mockRejectedValueOnce(new Error("This wallet has no open loan."));

    render(<RepayLoanModal isOpen loan={loan} onClose={vi.fn()} />);

    expect(await screen.findByText("This wallet has no open loan.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Repay" })).toBeDisabled();

    await userEvent.click(screen.getByRole("button", { name: /Retry/ }));
    await waitFor(() => expect(screen.getByTestId("repay-total")).toBeInTheDocument());
  });

  it("closes on Escape and renders nothing when closed", async () => {
    const onClose = vi.fn();
    const { rerender } = render(<RepayLoanModal isOpen loan={loan} onClose={onClose} />);
    await screen.findByTestId("repay-total");
    await userEvent.keyboard("{Escape}");
    expect(onClose).toHaveBeenCalled();

    rerender(<RepayLoanModal isOpen={false} loan={loan} onClose={onClose} />);
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });
});
