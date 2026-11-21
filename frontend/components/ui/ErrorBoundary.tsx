/**
 * Error Boundary Component
 * Catches and displays errors gracefully with RPC retry handling
 */

"use client";

import React, { Component, ReactNode, ErrorInfo } from "react";
import {
  AlertTriangle,
  RefreshCw,
  ChevronDown,
  Server,
  Copy,
  Check,
  Wifi,
  WifiOff,
} from "lucide-react";

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
  onRetry?: () => void;
  rpcNodes?: string[];
  onNodeSwitch?: (nodeUrl: string) => void;
}

interface State {
  hasError: boolean;
  error?: Error;
  errorInfo?: ErrorInfo;
  showDrawer: boolean;
  retryCount: number;
  isRetrying: boolean;
  activeNode: number;
  copied: boolean;
  suppressedCodes: number[];
}

const DEFAULT_RPC_NODES = [
  "https://soroban-testnet.stellar.org",
  "https://mainnet.sorobanrpc.com",
  "https://rpc.lightsail.network",
];

const MAX_AUTO_RETRIES = 3;

export class ErrorBoundary extends Component<Props, State> {
  private retryTimeout: ReturnType<typeof setTimeout> | null = null;
  private isMountedFlag = false;

  constructor(props: Props) {
    super(props);
    this.state = {
      hasError: false,
      showDrawer: false,
      retryCount: 0,
      isRetrying: false,
      activeNode: 0,
      copied: false,
      suppressedCodes: [],
    };
  }

  componentDidMount() {
    this.isMountedFlag = true;
  }

  componentWillUnmount() {
    this.isMountedFlag = false;
    if (this.retryTimeout) {
      clearTimeout(this.retryTimeout);
      this.retryTimeout = null;
    }
  }

  static getDerivedStateFromError(error: Error): Partial<State> {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error("Error caught by boundary:", error, errorInfo);

    if (!this.isMountedFlag) return;

    const isTransient = this.isTransientError(error);
    if (isTransient && this.state.retryCount < MAX_AUTO_RETRIES) {
      this.scheduleAutoRetry();
    } else {
      this.setState({ errorInfo });
    }
  }

  private isTransientError(error: Error): boolean {
    const msg = error.message.toLowerCase();
    if (
      msg.includes("504") ||
      msg.includes("gateway timeout") ||
      msg.includes("502") ||
      msg.includes("503") ||
      msg.includes("bad gateway") ||
      msg.includes("service unavailable")
    ) {
      return true;
    }
    if (msg.includes("network") || msg.includes("fetch failed")) return true;
    return false;
  }

  private scheduleAutoRetry() {
    const delay = Math.min(1000 * Math.pow(2, this.state.retryCount), 8000);
    this.setState({ isRetrying: true });

    this.retryTimeout = setTimeout(() => {
      if (!this.isMountedFlag) return;
      this.setState(
        (prev) => ({
          hasError: false,
          error: undefined,
          errorInfo: undefined,
          retryCount: prev.retryCount + 1,
          isRetrying: false,
        }),
        () => {
          this.props.onRetry?.();
        }
      );
    }, delay);
  }

  private handleManualRetry = () => {
    if (this.state.isRetrying) return;
    this.setState(
      { hasError: false, error: undefined, errorInfo: undefined, isRetrying: true },
      () => {
        this.props.onRetry?.();
        setTimeout(() => {
          if (this.isMountedFlag) this.setState({ isRetrying: false });
        }, 500);
      }
    );
  };

  private handleSwitchNode = (index: number) => {
    const nodes = this.props.rpcNodes ?? DEFAULT_RPC_NODES;
    const nodeUrl = nodes[index];
    if (!nodeUrl) return;
    this.setState({ activeNode: index, hasError: false, error: undefined, errorInfo: undefined });
    this.props.onNodeSwitch?.(nodeUrl);
  };

  private handleCopyError = () => {
    if (!this.state.error) return;
    const text = `${this.state.error.name}: ${this.state.error.message}\n${this.state.errorInfo?.componentStack ?? ""}`;
    navigator.clipboard.writeText(text).then(() => {
      this.setState({ copied: true });
      setTimeout(() => this.isMountedFlag && this.setState({ copied: false }), 2000);
    });
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      const nodes = this.props.rpcNodes ?? DEFAULT_RPC_NODES;
      const { error, errorInfo, showDrawer, isRetrying, retryCount, activeNode, copied } = this.state;

      return (
        <div className="min-h-screen flex items-center justify-center p-4">
          <div className="bg-[#0A0F11] border border-red-500/30 rounded-2xl p-8 max-w-md w-full text-center">
            {/* Retrying indicator */}
            {isRetrying ? (
              <>
                <div className="w-16 h-16 bg-[#33C5E0]/10 rounded-full flex items-center justify-center mx-auto mb-4">
                  <RefreshCw className="text-[#33C5E0] animate-spin" size={32} />
                </div>
                <h2 className="text-2xl font-bold text-white mb-2">Reconnecting…</h2>
                <p className="text-[#8899A6] mb-2">
                  Transient network error detected. Retrying automatically…
                </p>
                <p className="text-xs text-slate-500">Attempt {retryCount + 1} of {MAX_AUTO_RETRIES}</p>
              </>
            ) : (
              <>
                <div className="w-16 h-16 bg-red-500/10 rounded-full flex items-center justify-center mx-auto mb-4">
                  <AlertTriangle className="text-red-400" size={32} />
                </div>
                <h2 className="text-2xl font-bold text-white mb-2">Something went wrong</h2>
                <p className="text-[#8899A6] mb-6">
                  {error?.message || "An unexpected error occurred"}
                </p>

                {/* RPC Node Switcher */}
                <div className="mb-4 p-3 rounded-xl bg-white/[0.03] border border-white/[0.06] text-left">
                  <p className="text-xs text-slate-400 mb-2 flex items-center gap-1.5">
                    <Server size={12} />
                    RPC Node
                  </p>
                  <div className="space-y-1.5">
                    {nodes.map((node, i) => (
                      <button
                        key={node}
                        onClick={() => this.handleSwitchNode(i)}
                        className={`w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-[11px] transition-colors ${
                          i === activeNode
                            ? "bg-[#33C5E0]/10 text-[#33C5E0] border border-[#33C5E0]/20"
                            : "bg-white/[0.02] text-slate-400 border border-transparent hover:bg-white/[0.05]"
                        }`}
                      >
                        <span className="truncate font-mono">{node}</span>
                        {i === activeNode ? (
                          <Wifi size={11} className="flex-shrink-0 ml-2" />
                        ) : (
                          <WifiOff size={11} className="flex-shrink-0 ml-2 opacity-40" />
                        )}
                      </button>
                    ))}
                  </div>
                </div>

                {/* Action buttons */}
                <div className="flex gap-2 mb-4">
                  <button
                    onClick={this.handleManualRetry}
                    className="flex-1 bg-[#33C5E0] text-[#161E22] px-4 py-2.5 rounded-full font-medium flex items-center justify-center gap-2 hover:bg-[#2AB5D0] transition-colors text-sm"
                  >
                    <RefreshCw size={15} />
                    Retry
                  </button>
                  <button
                    onClick={() => window.location.reload()}
                    className="flex-1 bg-white/5 text-slate-300 px-4 py-2.5 rounded-full font-medium border border-white/10 hover:bg-white/10 transition-colors text-sm"
                  >
                    Reload Page
                  </button>
                </div>

                {/* Technical details drawer */}
                <div className="border-t border-white/[0.06] pt-3">
                  <button
                    onClick={() => this.setState({ showDrawer: !showDrawer })}
                    className="flex items-center gap-1.5 text-xs text-slate-500 hover:text-slate-300 transition-colors mx-auto"
                    aria-expanded={showDrawer}
                  >
                    Technical Details
                    <ChevronDown
                      size={12}
                      className={`transition-transform ${showDrawer ? "rotate-180" : ""}`}
                    />
                  </button>

                  {showDrawer && (
                    <div className="mt-3 text-left">
                      <div className="p-3 rounded-xl bg-black/40 border border-white/[0.06] font-mono text-[10px] text-slate-400 max-h-40 overflow-y-auto">
                        <p className="text-red-400">
                          {error?.name}: {error?.message}
                        </p>
                        {error?.stack && (
                          <pre className="mt-2 whitespace-pre-wrap text-slate-500 text-[9px]">
                            {error.stack.split("\n").slice(0, 5).join("\n")}
                          </pre>
                        )}
                        {errorInfo?.componentStack && (
                          <pre className="mt-2 whitespace-pre-wrap text-slate-600 text-[9px]">
                            {errorInfo.componentStack.split("\n").slice(0, 3).join("\n")}
                          </pre>
                        )}
                      </div>
                      <button
                        onClick={this.handleCopyError}
                        className="mt-2 flex items-center gap-1.5 text-[11px] text-slate-500 hover:text-slate-300 transition-colors"
                      >
                        {copied ? (
                          <>
                            <Check size={11} className="text-emerald-400" />
                            Copied!
                          </>
                        ) : (
                          <>
                            <Copy size={11} />
                            Copy error details
                          </>
                        )}
                      </button>
                    </div>
                  )}
                </div>
              </>
            )}
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

/**
 * Simple Error Display Component
 */
export function ErrorDisplay({
  error,
  onRetry,
}: {
  error: string;
  onRetry?: () => void;
}) {
  return (
    <div className="bg-red-500/10 border border-red-500/30 rounded-2xl p-6 text-center">
      <div className="w-12 h-12 bg-red-500/10 rounded-full flex items-center justify-center mx-auto mb-4">
        <AlertTriangle className="text-red-400" size={24} />
      </div>
      <p className="text-red-400 mb-4">{error}</p>
      {onRetry && (
        <button
          onClick={onRetry}
          className="bg-red-500/20 text-red-400 px-4 py-2 rounded-lg hover:bg-red-500/30 transition-colors"
        >
          Try Again
        </button>
      )}
    </div>
  );
}

/**
 * Empty State Component
 */
export function EmptyState({
  title,
  description,
  action,
}: {
  title: string;
  description: string;
  action?: {
    label: string;
    onClick: () => void;
  };
}) {
  return (
    <div className="bg-[#0A0F11] border border-[#161E22] rounded-2xl p-12 text-center">
      <h3 className="text-xl font-semibold text-white mb-2">{title}</h3>
      <p className="text-[#8899A6] mb-6 max-w-md mx-auto">{description}</p>
      {action && (
        <button
          onClick={action.onClick}
          className="bg-[#33C5E0] text-[#161E22] px-6 py-3 rounded-full font-medium hover:bg-[#2AB5D0] transition-colors"
        >
          {action.label}
        </button>
      )}
    </div>
  );
}
