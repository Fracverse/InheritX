/**
 * Lending Pool API Client
 */

export interface PoolState {
  total_deposits: string;
  total_borrowed: string;
  utilization_rate: number;
  current_apy: number;
  reserve_factor: number;
}

export interface UserLendingData {
  shares: string;
  underlying_balance: string;
  total_earnings: string;
  deposit_history: LendingTransaction[];
}

export interface LendingTransaction {
  id: string;
  type: 'DEPOSIT' | 'WITHDRAW';
  amount: string;
  timestamp: string;
  tx_hash: string;
  status: 'PENDING' | 'COMPLETED' | 'FAILED';
}

export class LendingAPI {
  private baseUrl: string;
  private getAuthToken: () => string | null;

  constructor(baseUrl: string = "", getAuthToken: () => string | null) {
    this.baseUrl = baseUrl;
    this.getAuthToken = getAuthToken;
  }

  private async request<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
    const token = this.getAuthToken();
    const headers: Record<string, string> = {
      "Content-Type": "application/json",
      ...((options.headers as Record<string, string>) || {}),
    };

    if (token) {
      headers["Authorization"] = `Bearer ${token}`;
    }

    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      ...options,
      headers,
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({}));
      throw new Error(error.error || `Request failed with status ${response.status}`);
    }

    return response.json();
  }

  async getPoolState(): Promise<PoolState> {
    return this.request<PoolState>("/api/lending/pool-state");
  }

  async getUserShares(address: string): Promise<UserLendingData> {
    return this.request<UserLendingData>(`/api/lending/shares/${address}`);
  }

  async getCurrentRate(): Promise<{ apy: number }> {
    return this.request<{ apy: number }>("/api/lending/current-rate");
  }

  async deposit(amount: string): Promise<{ tx_hash: string }> {
    return this.request<{ tx_hash: string }>("/api/lending/deposit", {
      method: "POST",
      body: JSON.stringify({ amount }),
    });
  }

  async withdraw(shares: string): Promise<{ tx_hash: string }> {
    return this.request<{ tx_hash: string }>("/api/lending/withdraw", {
      method: "POST",
      body: JSON.stringify({ shares }),
    });
  }
}

export function createLendingAPI(
  getAuthToken: () => string | null = () => localStorage.getItem("auth_token")
): LendingAPI {
  return new LendingAPI("", getAuthToken);
}

export default createLendingAPI;
