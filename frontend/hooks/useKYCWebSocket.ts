import { useEffect, useRef, useState } from "react";
import type { KYCStatus } from "@/app/lib/api/kyc";

interface KycUpdateEvent {
  wallet_address: string;
  kyc_status: string;
  event_type: string;
}

interface UseKYCWebSocketOptions {
  walletAddress: string | null;
  onStatusUpdate: (status: KYCStatus) => void;
  enabled?: boolean;
}

const WS_API_BASE = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080";
const WS_PATH = process.env.NEXT_PUBLIC_WS_PATH || "/ws/kyc";

function buildWsUrl(): string {
  const explicit = process.env.NEXT_PUBLIC_WS_URL;
  if (explicit) return explicit;
  return WS_API_BASE.replace(/^http/i, "ws").replace(/\/$/, "") + WS_PATH;
}

export function useKYCWebSocket({ walletAddress, onStatusUpdate, enabled = true }: UseKYCWebSocketOptions) {
  const [isConnected, setIsConnected] = useState(false);
  const [lastHeartbeat, setLastHeartbeat] = useState<number>(Date.now());
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimerRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttemptsRef = useRef(0);
  const shouldReconnectRef = useRef(true);

  useEffect(() => {
    if (typeof window === "undefined" || typeof WebSocket === "undefined") return;
    if (!enabled || !walletAddress) {
      // Clean up if disabled or no wallet
      if (wsRef.current) {
        shouldReconnectRef.current = false;
        wsRef.current.close();
        wsRef.current = null;
      }
      setIsConnected(false);
      return;
    }

    shouldReconnectRef.current = true;

    const connect = () => {
      if (wsRef.current?.readyState === WebSocket.OPEN || wsRef.current?.readyState === WebSocket.CONNECTING) {
        return;
      }

      const ws = new WebSocket(buildWsUrl());
      wsRef.current = ws;

      ws.onopen = () => {
        reconnectAttemptsRef.current = 0;
        setIsConnected(true);
        setLastHeartbeat(Date.now());
      };

      ws.onmessage = (msg) => {
        // Update heartbeat on any message
        setLastHeartbeat(Date.now());

        let data: unknown;
        try {
          data = JSON.parse(String(msg.data));
        } catch {
          return;
        }

        if (!data || typeof data !== "object") return;
        const ev = data as Record<string, unknown>;

        // KycUpdateEvent: { wallet_address, kyc_status, event_type }
        if (typeof ev.kyc_status === "string" && typeof ev.wallet_address === "string") {
          const event = ev as unknown as KycUpdateEvent;

          // Filter by wallet address
          if (!walletAddress || event.wallet_address.toLowerCase() !== walletAddress.toLowerCase()) {
            return;
          }

          const status = event.kyc_status.toLowerCase();
          if (status === "pending" || status === "submitted" || status === "approved" || status === "rejected") {
            onStatusUpdate(status as KYCStatus);
          }
        }
      };

      ws.onclose = () => {
        setIsConnected(false);
        wsRef.current = null;

        if (!shouldReconnectRef.current) return;

        // Exponential backoff with max 30s
        const delay = Math.min(30000, 1000 * 2 ** reconnectAttemptsRef.current++);
        reconnectTimerRef.current = setTimeout(connect, delay);
      };

      ws.onerror = () => {
        ws.close();
      };
    };

    connect();

    return () => {
      shouldReconnectRef.current = false;
      if (reconnectTimerRef.current) {
        clearTimeout(reconnectTimerRef.current);
        reconnectTimerRef.current = null;
      }
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
      }
      setIsConnected(false);
    };
  }, [walletAddress, onStatusUpdate, enabled]);

  return {
    isConnected,
    lastHeartbeat,
    isHealthy: isConnected && (Date.now() - lastHeartbeat < 60000), // Healthy if connected and heartbeat within last 60s
  };
}
