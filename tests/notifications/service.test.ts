import { describe, it, expect, beforeEach } from "vitest";
import { notificationService } from "@/lib/notifications/service";
import { NotificationRequest } from "@/lib/notifications/types";

describe("Notification Service", () => {
  const testUserId = "user_123";

  beforeEach(() => {
    // Reset service state if needed
  });

  describe("send", () => {
    it("should send email notification", async () => {
      const request: NotificationRequest = {
        user_id: testUserId,
        category: "plan_created",
        data: {
          user_name: "John Doe",
          plan_name: "Family Trust",
          plan_id: "plan_123",
          total_assets: "50000",
          beneficiaries_count: "3",
        },
        recipient: {
          email: "john@example.com",
        },
      };

      const results = await notificationService.send(request);

      expect(results).toHaveLength(2); // email and in_app by default
      expect(results[0].success).toBe(true);
      expect(results[0].type).toBe("email");
      expect(results[0].status).toBe("delivered");
    });

    it("should send multiple notification types", async () => {
      // Enable all notification types
      notificationService.updatePreferences(testUserId, {
        user_id: testUserId,
        email_enabled: true,
        sms_enabled: true,
        push_enabled: true,
        in_app_enabled: true,
        categories: {},
      });

      const request: NotificationRequest = {
        user_id: testUserId,
        category: "claim_approved",
        data: {
          user_name: "Jane Smith",
          claim_id: "claim_456",
          claim_amount: "15000",
          approved_at: new Date().toISOString(),
        },
        recipient: {
          email: "jane@example.com",
          phone: "+1234567890",
          device_token: "device_token_123",
        },
      };

      const results = await notificationService.send(request);

      expect(results.length).toBeGreaterThan(1);
      expect(results.every((r) => r.success)).toBe(true);
    });

    it("should handle missing template", async () => {
      const request: NotificationRequest = {
        user_id: testUserId,
        category: "invalid_category" as any,
        data: {},
      };

      await expect(notificationService.send(request)).rejects.toThrow(
        "Template not found"
      );
    });
  });

  describe("preferences", () => {
    it("should get default preferences", () => {
      const prefs = notificationService.getPreferences("new_user");

      expect(prefs.user_id).toBe("new_user");
      expect(prefs.email_enabled).toBe(true);
      expect(prefs.in_app_enabled).toBe(true);
    });

    it("should update preferences", () => {
      const updated = notificationService.updatePreferences(testUserId, {
        email_enabled: false,
        sms_enabled: true,
      });

      expect(updated.email_enabled).toBe(false);
      expect(updated.sms_enabled).toBe(true);
    });

    it("should respect category preferences", async () => {
      notificationService.updatePreferences(testUserId, {
        user_id: testUserId,
        email_enabled: true,
        categories: {
          plan_created: {
            email: false,
            sms: false,
            push: false,
            in_app: true,
          },
        },
      });

      const request: NotificationRequest = {
        user_id: testUserId,
        category: "plan_created",
        data: {
          user_name: "Test User",
          plan_name: "Test Plan",
          plan_id: "plan_789",
          total_assets: "10000",
          beneficiaries_count: "1",
        },
      };

      const results = await notificationService.send(request);

      // Should only send in_app
      expect(results).toHaveLength(1);
      expect(results[0].type).toBe("in_app");
    });
  });

  describe("getUserNotifications", () => {
    it("should get user notifications", async () => {
      // Send a notification first
      await notificationService.send({
        user_id: testUserId,
        category: "plan_created",
        data: {
          user_name: "Test",
          plan_name: "Test",
          plan_id: "test",
          total_assets: "0",
          beneficiaries_count: "0",
        },
      });

      const notifications = notificationService.getUserNotifications(testUserId);

      expect(notifications.length).toBeGreaterThan(0);
      expect(notifications[0].user_id).toBe(testUserId);
    });

    it("should filter by type", async () => {
      const notifications = notificationService.getUserNotifications(testUserId, {
        type: "email",
      });

      expect(notifications.every((n) => n.type === "email")).toBe(true);
    });

    it("should filter by status", async () => {
      const notifications = notificationService.getUserNotifications(testUserId, {
        status: "delivered",
      });

      expect(notifications.every((n) => n.status === "delivered")).toBe(true);
    });
  });

  describe("markAsRead", () => {
    it("should mark notification as read", async () => {
      const results = await notificationService.send({
        user_id: testUserId,
        category: "plan_created",
        data: {
          user_name: "Test",
          plan_name: "Test",
          plan_id: "test",
          total_assets: "0",
          beneficiaries_count: "0",
        },
      });

      const notificationId = results[0].notification_id;
      const success = notificationService.markAsRead(notificationId);

      expect(success).toBe(true);

      const notifications = notificationService.getUserNotifications(testUserId);
      const notification = notifications.find((n) => n.id === notificationId);

      expect(notification?.status).toBe("read");
      expect(notification?.read_at).toBeDefined();
    });

    it("should return false for non-existent notification", () => {
      const success = notificationService.markAsRead("non_existent");
      expect(success).toBe(false);
    });
  });

  describe("retry", () => {
    it("should retry failed notification", async () => {
      const results = await notificationService.send({
        user_id: testUserId,
        category: "plan_created",
        data: {
          user_name: "Test",
          plan_name: "Test",
          plan_id: "test",
          total_assets: "0",
          beneficiaries_count: "0",
        },
      });

      const notificationId = results[0].notification_id;
      const retryResult = await notificationService.retry(notificationId);

      expect(retryResult.success).toBe(true);
      expect(retryResult.notification_id).toBe(notificationId);
    });

    it("should throw error for non-existent notification", async () => {
      await expect(notificationService.retry("non_existent")).rejects.toThrow(
        "Notification not found"
      );
    });
  });
});
