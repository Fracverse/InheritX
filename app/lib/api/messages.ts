/**
 * Legacy Messages API Client
 * 
 * Provides type-safe methods for interacting with the legacy messages system
 */

export type MessageStatus = 'DRAFT' | 'FINALIZED' | 'UNLOCKED';

export interface LegacyMessage {
  id: string;
  vault_id: string;
  title: string;
  content_encrypted: string;
  unlock_at: string;
  status: MessageStatus;
  created_at: string;
  updated_at: string;
  beneficiary_ids: string[];
}

export interface MessageAuditLog {
  id: string;
  message_id: string;
  action: 'CREATE' | 'UPDATE' | 'FINALIZE' | 'UNLOCK' | 'ACCESS' | 'DELETE';
  performed_by: string;
  timestamp: string;
  ip_address?: string;
}

export class MessagesAPI {
  private baseUrl: string;
  private getAuthToken: () => string | null;

  constructor(baseUrl: string = "", getAuthToken: () => string | null) {
    this.baseUrl = baseUrl;
    this.getAuthToken = getAuthToken;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {},
  ): Promise<T> {
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
      throw new Error(
        error.error || `Request failed with status ${response.status}`,
      );
    }

    return response.json();
  }

  /**
   * Create encrypted message
   */
  async createMessage(data: {
    vault_id: string;
    title: string;
    content: string; // Will be encrypted client-side or handled by middleware
    unlock_at: string;
    beneficiary_ids: string[];
  }): Promise<LegacyMessage> {
    return this.request<LegacyMessage>("/api/messages/create", {
      method: "POST",
      body: JSON.stringify(data),
    });
  }

  /**
   * Update message (only if not finalized)
   */
  async updateMessage(id: string, data: Partial<LegacyMessage>): Promise<LegacyMessage> {
    return this.request<LegacyMessage>(`/api/messages/${id}`, {
      method: "PUT",
      body: JSON.stringify(data),
    });
  }

  /**
   * Finalize message (locks it)
   */
  async finalizeMessage(id: string): Promise<{ success: boolean }> {
    return this.request<{ success: boolean }>(`/api/messages/${id}/finalize`, {
      method: "POST",
    });
  }

  /**
   * Delete message
   */
  async deleteMessage(id: string): Promise<{ success: boolean }> {
    return this.request<{ success: boolean }>(`/api/messages/${id}`, {
      method: "DELETE",
    });
  }

  /**
   * Get message metadata
   */
  async getMessage(id: string): Promise<LegacyMessage> {
    return this.request<LegacyMessage>(`/api/messages/${id}`);
  }

  /**
   * Get all messages for a vault
   */
  async getVaultMessages(vaultId: string): Promise<LegacyMessage[]> {
    return this.request<LegacyMessage[]>(`/api/messages/vault/${vaultId}`);
  }

  /**
   * Unlock message (for beneficiaries)
   */
  async unlockMessage(id: string): Promise<{ content: string }> {
    return this.request<{ content: string }>(`/api/messages/${id}/unlock`, {
      method: "POST",
    });
  }

  /**
   * Get access audit trail
   */
  async getAccessAudit(id: string): Promise<MessageAuditLog[]> {
    return this.request<MessageAuditLog[]>(`/api/messages/${id}/access-audit`);
  }
}

export function createMessagesAPI(
  getAuthToken: () => string | null = () => localStorage.getItem("auth_token"),
): MessagesAPI {
  return new MessagesAPI("", getAuthToken);
}

export default createMessagesAPI;
