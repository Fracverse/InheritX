import type { LoanPosition, RepaymentQuote } from "@/app/lib/stellar/lendingClient";

/**
 * In-memory loan for frontend demo mode (`NEXT_PUBLIC_USE_MOCK_DATA=true`).
 * Interest follows the contract's simple-interest formula so the demo payoff
 * moves the same way the on-chain one does.
 */

const SECONDS_IN_YEAR = 31_536_000;
const UNIT = BigInt(10_000_000); // 7 decimals

function createDemoLoan(borrower: string): LoanPosition {
  const now = Math.floor(Date.now() / 1000);
  return {
    loanId: BigInt(1),
    borrower,
    asset: "USDC",
    principal: BigInt(6_200) * UNIT,
    collateralAmount: BigInt(10_000) * UNIT,
    collateralToken: "XLM",
    borrowTime: now - 45 * 86_400,
    dueDate: now + 15 * 86_400,
    interestRateBps: 800,
  };
}

let demoLoan: LoanPosition | null | undefined;

function accruedInterest(loan: LoanPosition, nowSeconds: number): bigint {
  const elapsed = BigInt(Math.max(0, nowSeconds - loan.borrowTime));
  const numerator = loan.principal * BigInt(loan.interestRateBps) * elapsed;
  const denominator = BigInt(10_000 * SECONDS_IN_YEAR);
  return (numerator + denominator / BigInt(2)) / denominator;
}

export const mockLoans = {
  getLoan(borrower: string): LoanPosition | null {
    if (demoLoan === undefined) demoLoan = createDemoLoan(borrower);
    return demoLoan ? { ...demoLoan, borrower } : null;
  },

  getRepaymentQuote(borrower: string): RepaymentQuote {
    const loan = this.getLoan(borrower);
    if (!loan) throw new Error("This wallet has no open loan.");
    const interest = accruedInterest(loan, Math.floor(Date.now() / 1000));
    return { principal: loan.principal, interest, lateFee: BigInt(0), total: loan.principal + interest };
  },

  repay(borrower: string): { hash: string; quote: RepaymentQuote } {
    const quote = this.getRepaymentQuote(borrower);
    demoLoan = null;
    return { hash: `demo-${Date.now().toString(16)}`, quote };
  },

  /** Test helper: restores the initial demo loan. */
  reset(): void {
    demoLoan = undefined;
  },
};
