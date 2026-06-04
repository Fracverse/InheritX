/**
 * React hook for notifications
 */
import { useState, useEffect, useCallback } from "react";
import {
  Notification,
  NotificationPreferences,
  NotificationType,
  NotificationStatus,
  NotificationCategory,
} from "@/lib/notifications/types";

interface UseNotificationsOptions {
  user_id: string;
  type?: NotificationType;
  status?: NotificationStatus;
  category?: NotificationCategory;
  autoRefresh?: boolean;
  refreshInterval?: number;
}

export function useNotifications(options: UseNotificationsOptions) {
  const {
    user_id,
    type,
    status,
    category,
    autoRefresh = false,
    refreshInterval = 30000,
  } = options;

  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [unreadCount, setUnreadCount] = useState(0);

  const fetchNotifications = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const params = new URLSearchParams({ user_id });
      if (type) params.set("type", type);
      if (status) params.set("status", status);
      if (category) params.set("category", category);

      const response = await fetch(`/api/notifications?${params}`);
      const result = await response.json();

      if (result.status === "ok") {
        setNotifications(result.data);
        setUnreadCount(
          result.data.filter((n: Notification) => n.status !== "read").length
        );
      } else {
        throw new Error(result.message || "Failed to fetch notifications");
      }
    } catch (err) {
      setError(err instanceof Error ? err : new Error("Unknown error"));
    } finally {
      setLoading(false);
    }
  }, [user_id, type, status, category]);

  useEffect(() => {
    fetchNotifications();

    if (autoRefresh) {
      const interval = setInterval(fetchNotifications, refreshInterval);
      return () => clearInterval(interval);
    }
  }, [fetchNotifications, autoRefresh, refreshInterval]);

  const markAsRead = useCallback(async (notification_id: string) => {
    try {
      const response = await fetch(`/api/notifications/${notification_id}/read`, {
        method: "PUT",
      });

      const result = await response.json();

      if (result.status === "ok") {
        setNotifications((prev) =>
          prev.map((n) =>
            n.id === notification_id ? { ...n, status: "read" as const } : n
          )
        );
        setUnreadCount((prev) => Math.max(0, prev - 1));
        return true;
      }

      return false;
    } catch (err) {
      console.error("Failed to mark as read:", err);
      return false;
    }
  }, []);

  const markAllAsRead = useCallback(async () => {
    const unread = notifications.filter((n) => n.status !== "read");
    await Promise.all(unread.map((n) => markAsRead(n.id)));
  }, [notifications, markAsRead]);

  return {
    notifications,
    loading,
    error,
    unreadCount,
    refetch: fetchNotifications,
    markAsRead,
    markAllAsRead,
  };
}

export function useNotificationPreferences(user_id: string) {
  const [preferences, setPreferences] = useState<NotificationPreferences | null>(
    null
  );
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const fetchPreferences = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const response = await fetch(`/api/notifications/preferences/${user_id}`);
      const result = await response.json();

      if (result.status === "ok") {
        setPreferences(result.data);
      } else {
        throw new Error(result.message || "Failed to fetch preferences");
      }
    } catch (err) {
      setError(err instanceof Error ? err : new Error("Unknown error"));
    } finally {
      setLoading(false);
    }
  }, [user_id]);

  useEffect(() => {
    fetchPreferences();
  }, [fetchPreferences]);

  const updatePreferences = useCallback(
    async (updates: Partial<NotificationPreferences>) => {
      setLoading(true);
      setError(null);

      try {
        const response = await fetch(
          `/api/notifications/preferences/${user_id}`,
          {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(updates),
          }
        );

        const result = await response.json();

        if (result.status === "ok") {
          setPreferences(result.data);
          return true;
        } else {
          throw new Error(result.message || "Failed to update preferences");
        }
      } catch (err) {
        setError(err instanceof Error ? err : new Error("Unknown error"));
        return false;
      } finally {
        setLoading(false);
      }
    },
    [user_id]
  );

  return {
    preferences,
    loading,
    error,
    updatePreferences,
    refetch: fetchPreferences,
  };
}
