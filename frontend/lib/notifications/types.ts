/**
 * Notification System Types
 */

export type NotificationType = 
  | "email" 
  | "sms" 
  | "push" 
  | "in_app";

export type NotificationPriority = "low" | "normal" | "high" | "urgent";

export type NotificationStatus = 
  | "pending" 
  | "sent" 
  | "delivered" 
  | "failed" 
  | "read";

export type NotificationCategory =
  | "plan_created"
  | "plan_updated"
  | "claim_submitted"
  | "claim_approved"
  | "claim_rejected"
  | "message_unlocked"
  | "emergency_activated"
  | "will_generated"
  | "beneficiary_added"
  | "security_alert"
  | "payment_received"
  | "kyc_verified"
  | "system_update";

export interface NotificationPreferences {
  user_id: string;
  email_enabled: boolean;
  sms_enabled: boolean;
  push_enabled: boolean;
  in_app_enabled: boolean;
  categories: {
    [key in NotificationCategory]?: {
      email: boolean;
      sms: boolean;
      push: boolean;
      in_app: boolean;
    };
  };
  quiet_hours?: {
    enabled: boolean;
    start: string; // HH:mm format
    end: string;
    timezone: string;
  };
  frequency?: {
    digest_enabled: boolean;
    digest_frequency: "daily" | "weekly";
    digest_time: string;
  };
}

export interface NotificationTemplate {
  id: string;
  category: NotificationCategory;
  type: NotificationType;
  subject?: string; // For email
  title: string; // For push/in-app
  body: string;
  variables: string[]; // Template variables like {{user_name}}
  created_at: string;
  updated_at: string;
}

export interface Notification {
  id: string;
  user_id: string;
  type: NotificationType;
  category: NotificationCategory;
  priority: NotificationPriority;
  status: NotificationStatus;
  subject?: string;
  title: string;
  body: string;
  data?: Record<string, any>; // Additional metadata
  recipient: {
    email?: string;
    phone?: string;
    device_token?: string;
  };
  delivery_attempts: number;
  last_attempt_at?: string;
  sent_at?: string;
  delivered_at?: string;
  read_at?: string;
  error_message?: string;
  created_at: string;
  updated_at: string;
}

export interface NotificationRequest {
  user_id: string;
  category: NotificationCategory;
  priority?: NotificationPriority;
  data: Record<string, any>;
  recipient?: {
    email?: string;
    phone?: string;
    device_token?: string;
  };
  send_immediately?: boolean;
  scheduled_for?: string;
}

export interface NotificationDeliveryResult {
  success: boolean;
  notification_id: string;
  type: NotificationType;
  status: NotificationStatus;
  error?: string;
  delivered_at?: string;
}

export interface NotificationStats {
  total_sent: number;
  total_delivered: number;
  total_failed: number;
  total_read: number;
  delivery_rate: number;
  read_rate: number;
  by_type: {
    [key in NotificationType]?: {
      sent: number;
      delivered: number;
      failed: number;
    };
  };
  by_category: {
    [key in NotificationCategory]?: {
      sent: number;
      delivered: number;
      failed: number;
    };
  };
}
