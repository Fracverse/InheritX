/**
 * Notification Service
 * Handles sending notifications via email, SMS, and push
 */
import {
  Notification,
  NotificationRequest,
  NotificationDeliveryResult,
  NotificationPreferences,
  NotificationCategory,
  NotificationType,
  NotificationStatus,
} from "./types";
import { notificationTemplates, renderTemplate } from "./templates";

class NotificationService {
  private notifications: Map<string, Notification> = new Map();
  private preferences: Map<string, NotificationPreferences> = new Map();

  /**
   * Send a notification
   */
  async send(request: NotificationRequest): Promise<NotificationDeliveryResult[]> {
    const results: NotificationDeliveryResult[] = [];

    // Get user preferences
    const prefs = this.getPreferences(request.user_id);

    // Get template
    const template = notificationTemplates[request.category];
    if (!template) {
      throw new Error(`Template not found for category: ${request.category}`);
    }

    // Render template
    const rendered = renderTemplate(template, request.data);

    // Determine which notification types to send based on preferences
    const typesToSend = this.getEnabledTypes(prefs, request.category);

    // Check quiet hours
    if (this.isQuietHours(prefs) && request.priority !== "urgent") {
      // Queue for later
      console.log("Quiet hours active, queueing notification");
    }

    // Send each type
    for (const type of typesToSend) {
      const notification = this.createNotification(
        request,
        type,
        rendered,
        template
      );

      try {
        const result = await this.sendByType(notification);
        results.push(result);
        this.notifications.set(notification.id, notification);
      } catch (error) {
        results.push({
          success: false,
          notification_id: notification.id,
          type,
          status: "failed",
          error: error instanceof Error ? error.message : "Unknown error",
        });
      }
    }

    return results;
  }

  /**
   * Send notification by type
   */
  private async sendByType(
    notification: Notification
  ): Promise<NotificationDeliveryResult> {
    switch (notification.type) {
      case "email":
        return this.sendEmail(notification);
      case "sms":
        return this.sendSMS(notification);
      case "push":
        return this.sendPush(notification);
      case "in_app":
        return this.sendInApp(notification);
      default:
        throw new Error(`Unsupported notification type: ${notification.type}`);
    }
  }

  /**
   * Send email notification
   */
  private async sendEmail(
    notification: Notification
  ): Promise<NotificationDeliveryResult> {
    // In production, integrate with email service (SendGrid, AWS SES, etc.)
    console.log("Sending email:", {
      to: notification.recipient.email,
      subject: notification.subject,
      body: notification.body,
    });

    // Simulate email sending
    await this.delay(100);

    const now = new Date().toISOString();
    notification.status = "delivered";
    notification.sent_at = now;
    notification.delivered_at = now;
    notification.delivery_attempts += 1;

    return {
      success: true,
      notification_id: notification.id,
      type: "email",
      status: "delivered",
      delivered_at: now,
    };
  }

  /**
   * Send SMS notification
   */
  private async sendSMS(
    notification: Notification
  ): Promise<NotificationDeliveryResult> {
    // In production, integrate with SMS service (Twilio, AWS SNS, etc.)
    console.log("Sending SMS:", {
      to: notification.recipient.phone,
      message: notification.body,
    });

    // Simulate SMS sending
    await this.delay(100);

    const now = new Date().toISOString();
    notification.status = "delivered";
    notification.sent_at = now;
    notification.delivered_at = now;
    notification.delivery_attempts += 1;

    return {
      success: true,
      notification_id: notification.id,
      type: "sms",
      status: "delivered",
      delivered_at: now,
    };
  }

  /**
   * Send push notification
   */
  private async sendPush(
    notification: Notification
  ): Promise<NotificationDeliveryResult> {
    // In production, integrate with push service (Firebase, OneSignal, etc.)
    console.log("Sending push:", {
      to: notification.recipient.device_token,
      title: notification.title,
      body: notification.body,
    });

    // Simulate push sending
    await this.delay(100);

    const now = new Date().toISOString();
    notification.status = "delivered";
    notification.sent_at = now;
    notification.delivered_at = now;
    notification.delivery_attempts += 1;

    return {
      success: true,
      notification_id: notification.id,
      type: "push",
      status: "delivered",
      delivered_at: now,
    };
  }

  /**
   * Send in-app notification
   */
  private async sendInApp(
    notification: Notification
  ): Promise<NotificationDeliveryResult> {
    // Store in database for in-app display
    console.log("Creating in-app notification:", {
      user_id: notification.user_id,
      title: notification.title,
      body: notification.body,
    });

    const now = new Date().toISOString();
    notification.status = "sent";
    notification.sent_at = now;
    notification.delivery_attempts += 1;

    return {
      success: true,
      notification_id: notification.id,
      type: "in_app",
      status: "sent",
      delivered_at: now,
    };
  }

  /**
   * Create notification object
   */
  private createNotification(
    request: NotificationRequest,
    type: NotificationType,
    rendered: { subject?: string; title: string; body: string },
    template: any
  ): Notification {
    const id = `notif_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    return {
      id,
      user_id: request.user_id,
      type,
      category: request.category,
      priority: request.priority || "normal",
      status: "pending",
      subject: rendered.subject,
      title: rendered.title,
      body: rendered.body,
      data: request.data,
      recipient: request.recipient || {},
      delivery_attempts: 0,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
  }

  /**
   * Get enabled notification types for a category
   */
  private getEnabledTypes(
    prefs: NotificationPreferences,
    category: NotificationCategory
  ): NotificationType[] {
    const types: NotificationType[] = [];

    const categoryPrefs = prefs.categories[category];

    if (prefs.email_enabled && categoryPrefs?.email !== false) {
      types.push("email");
    }
    if (prefs.sms_enabled && categoryPrefs?.sms !== false) {
      types.push("sms");
    }
    if (prefs.push_enabled && categoryPrefs?.push !== false) {
      types.push("push");
    }
    if (prefs.in_app_enabled && categoryPrefs?.in_app !== false) {
      types.push("in_app");
    }

    return types;
  }

  /**
   * Check if currently in quiet hours
   */
  private isQuietHours(prefs: NotificationPreferences): boolean {
    if (!prefs.quiet_hours?.enabled) return false;

    const now = new Date();
    const currentTime = `${now.getHours().toString().padStart(2, "0")}:${now.getMinutes().toString().padStart(2, "0")}`;

    const { start, end } = prefs.quiet_hours;

    // Simple time comparison (doesn't handle timezone properly in this mock)
    return currentTime >= start && currentTime <= end;
  }

  /**
   * Get user notification preferences
   */
  getPreferences(user_id: string): NotificationPreferences {
    if (!this.preferences.has(user_id)) {
      // Return default preferences
      return {
        user_id,
        email_enabled: true,
        sms_enabled: false,
        push_enabled: true,
        in_app_enabled: true,
        categories: {},
      };
    }

    return this.preferences.get(user_id)!;
  }

  /**
   * Update user notification preferences
   */
  updatePreferences(
    user_id: string,
    prefs: Partial<NotificationPreferences>
  ): NotificationPreferences {
    const existing = this.getPreferences(user_id);
    const updated = { ...existing, ...prefs, user_id };
    this.preferences.set(user_id, updated);
    return updated;
  }

  /**
   * Mark notification as read
   */
  markAsRead(notification_id: string): boolean {
    const notification = this.notifications.get(notification_id);
    if (!notification) return false;

    notification.status = "read";
    notification.read_at = new Date().toISOString();
    notification.updated_at = new Date().toISOString();

    return true;
  }

  /**
   * Get user notifications
   */
  getUserNotifications(
    user_id: string,
    filters?: {
      type?: NotificationType;
      status?: NotificationStatus;
      category?: NotificationCategory;
    }
  ): Notification[] {
    let notifications = Array.from(this.notifications.values()).filter(
      (n) => n.user_id === user_id
    );

    if (filters?.type) {
      notifications = notifications.filter((n) => n.type === filters.type);
    }
    if (filters?.status) {
      notifications = notifications.filter((n) => n.status === filters.status);
    }
    if (filters?.category) {
      notifications = notifications.filter((n) => n.category === filters.category);
    }

    return notifications.sort(
      (a, b) =>
        new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
    );
  }

  /**
   * Retry failed notification
   */
  async retry(notification_id: string): Promise<NotificationDeliveryResult> {
    const notification = this.notifications.get(notification_id);
    if (!notification) {
      throw new Error("Notification not found");
    }

    notification.status = "pending";
    notification.last_attempt_at = new Date().toISOString();

    return this.sendByType(notification);
  }

  /**
   * Helper: delay
   */
  private delay(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}

// Export singleton instance
export const notificationService = new NotificationService();
