/**
 * Emergency Access API Client
 * 
 * Provides type-safe methods for interacting with the emergency access system
 */

export interface EmergencyContact {
  id: string;
  name: string;
  email: string;
  wallet_address: string;
  added_at: string;
}

export interface Guardian {
  id: string;
  name: string;
  wallet_address: string;
  is_approved: boolean;
}

export interface EmergencyStatus {
  is_active: boolean;
  activated_by?: string;
  activated_at?: string;
  expires_at?: string;
  cooldown_until?: string;
  withdrawal_limit_percent: number;
}

export interface EmergencyRequest {
  id: string;
  requester_address: string;
  requester_name: string;
  plan_id: string;
  status: 'PENDING' | 'APPROVED' | 'REJECTED';
  created_at: string;
  approvals_count: number;
  threshold_required: number;
}

export interface AuditLog {
  id: string;
  action: string;
  performed_by: string;
  timestamp: string;
  details?: string;
}

export class EmergencyAPI {
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
    // In a real app, we might check if token exists, 
    // but for this implementation we'll handle mock/real transitions gracefully.
    
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
   * Activate emergency access
   */
  async activateEmergency(planId: string): Promise<{ status: string }> {
    return this.request<{ status: string }>("/api/emergency/activate", {
      method: "POST",
      body: JSON.stringify({ plan_id: planId }),
    });
  }

  /**
   * Add emergency contact
   */
  async addContact(planId: string, contact: Omit<EmergencyContact, 'id' | 'added_at'>): Promise<EmergencyContact> {
    return this.request<EmergencyContact>("/api/emergency/contacts", {
      method: "POST",
      body: JSON.stringify({ plan_id: planId, ...contact }),
    });
  }

  /**
   * Remove emergency contact
   */
  async removeContact(contactId: string): Promise<{ success: boolean }> {
    return this.request<{ success: boolean }>(`/api/emergency/contacts/${contactId}`, {
      method: "DELETE",
    });
  }

  /**
   * List emergency contacts
   */
  async listContacts(planId: string): Promise<EmergencyContact[]> {
    return this.request<EmergencyContact[]>(`/api/emergency/contacts/${planId}`);
  }

  /**
   * Set guardians and threshold
   */
  async setGuardians(planId: string, guardians: string[], threshold: number): Promise<{ success: boolean }> {
    return this.request<{ success: boolean }>("/api/emergency/guardians", {
      method: "POST",
      body: JSON.stringify({ plan_id: planId, guardians, threshold }),
    });
  }

  /**
   * Approve emergency request
   */
  async approveRequest(requestId: string): Promise<{ success: boolean }> {
    return this.request<{ success: boolean }>("/api/emergency/approve", {
      method: "POST",
      body: JSON.stringify({ request_id: requestId }),
    });
  }

  /**
   * Revoke emergency access
   */
  async revokeAccess(planId: string): Promise<{ success: boolean }> {
    return this.request<{ success: boolean }>("/api/emergency/revoke", {
      method: "POST",
      body: JSON.stringify({ plan_id: planId }),
    });
  }

  /**
   * Get audit logs
   */
  async getAuditLogs(planId: string): Promise<AuditLog[]> {
    return this.request<AuditLog[]>("/api/emergency/audit-logs", {
      // Typically we'd pass plan_id as query param or in body if POST
      // assuming query param for now or endpoint handles it
    });
  }
}

export function createEmergencyAPI(
  getAuthToken: () => string | null = () => localStorage.getItem("auth_token"),
): EmergencyAPI {
  return new EmergencyAPI("", getAuthToken);
}

export default createEmergencyAPI;
