/**
 * Loan Service for InheritX
 * Handles all interactions with the loan backend APIs and contracts.
 */

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080/api";

export interface Loan {
  id: string;
  borrower: string;
  collateralAsset: string;
  collateralAmount: string;
  borrowedAmount: string;
  interestRate: number;
  duration: number;
  startTime: number;
  status: "ACTIVE" | "REPAID" | "LIQUIDATED";
  healthFactor: number;
  repaymentAmount: string;
}

export interface LoanSimulation {
  borrowedAmount: string;
  collateralRequired: string;
  interestRate: number;
  healthFactor: number;
  liquidationPrice: number;
}

export const loanService = {
  /**
   * Create a new loan
   */
  createLoan: async (data: {
    collateralAsset: string;
    collateralAmount: string;
    borrowAmount: string;
    duration: number;
    address: string;
  }): Promise<{ success: boolean; loanId?: string; error?: string }> => {
    try {
      const response = await fetch(`${API_BASE_URL}/loans/create`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
      });
      return await response.json();
    } catch (error) {
      console.error("Error creating loan:", error);
      return { success: false, error: "Failed to connect to server" };
    }
  },

  /**
   * Repay an existing loan
   */
  repayLoan: async (loanId: string): Promise<{ success: boolean; error?: string }> => {
    try {
      const response = await fetch(`${API_BASE_URL}/loans/${loanId}/repay`, {
        method: "POST",
      });
      return await response.json();
    } catch (error) {
      console.error("Error repaying loan:", error);
      return { success: false, error: "Failed to connect to server" };
    }
  },

  /**
   * Get details of a specific loan
   */
  getLoanDetails: async (loanId: string): Promise<Loan | null> => {
    try {
      const response = await fetch(`${API_BASE_URL}/loans/${loanId}`);
      if (!response.ok) return null;
      return await response.json();
    } catch (error) {
      console.error("Error fetching loan details:", error);
      return null;
    }
  },

  /**
   * Get all loans for a specific user address
   */
  getUserLoans: async (address: string): Promise<Loan[]> => {
    try {
      const response = await fetch(`${API_BASE_URL}/loans/user/${address}`);
      if (!response.ok) return [];
      return await response.json();
    } catch (error) {
      console.error("Error fetching user loans:", error);
      return [];
    }
  },

  /**
   * Simulate loan terms before creation
   */
  simulateLoan: async (data: {
    collateralAsset: string;
    collateralAmount: string;
    borrowAmount: string;
    duration: number;
  }): Promise<LoanSimulation | null> => {
    try {
      const response = await fetch(`${API_BASE_URL}/loans/simulate`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
      });
      if (!response.ok) return null;
      return await response.json();
    } catch (error) {
      console.error("Error simulating loan:", error);
      return null;
    }
  },

  /**
   * Get the current health factor for a loan
   */
  getHealthFactor: async (loanId: string): Promise<number> => {
    try {
      const response = await fetch(`${API_BASE_URL}/loans/${loanId}/health-factor`);
      if (!response.ok) return 0;
      const data = await response.json();
      return data.healthFactor;
    } catch (error) {
      console.error("Error fetching health factor:", error);
      return 0;
    }
  },
};
