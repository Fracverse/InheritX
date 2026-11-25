import { apiClient } from "./client";

export interface AnchorFeeQuote {
  anchor_id: string;
  anchor_name: string;
  currency: string;
  fee: number;
  fee_currency: string;
  sep24_supported: boolean;
  sep31_supported: boolean;
}

export class AnchorAPI {
  async getFees(currency?: string): Promise<AnchorFeeQuote[]> {
    const query = currency ? `?currency=${encodeURIComponent(currency)}` : "";
    return apiClient.get<AnchorFeeQuote[]>(`/api/anchor/fees${query}`);
  }
}

export const anchorAPI = new AnchorAPI();
