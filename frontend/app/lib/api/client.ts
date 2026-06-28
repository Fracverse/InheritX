/**
 * API Client Service for InheritX
 * Centralized API client with authentication, error handling, retry policies,
 * request timeouts, and signature-based auth for Stellar wallet users.
 */

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080";

/** Default timeout for API requests (30 seconds) */
const DEFAULT_TIMEOUT_MS = 30_000;

/** Default retry configuration */
const DEFAULT_RETRY_CONFIG: RetryConfig = {
  maxRetries: 3,
  baseDelayMs: 1000,
  maxDelayMs: 10_000,
  retryOnStatuses: [408, 429, 500, 502, 503, 504],
};

export interface ApiResponse<T> {
  status: string;
  data?: T;
  message?: string;
  error?: string;
}

export interface PaginatedResponse<T> {
  status: string;
  data: T[];
  page: number;
  limit: number;
  total_count: number;
  total_pages: number;
  has_next: boolean;
  has_prev: boolean;
}

/** Configuration for exponential backoff retries */
export interface RetryConfig {
  /** Maximum number of retry attempts (default: 3) */
  maxRetries: number;
  /** Base delay in milliseconds before first retry (default: 1000) */
  baseDelayMs: number;
  /** Maximum delay cap in milliseconds (default: 10000) */
  maxDelayMs: number;
  /** HTTP status codes that should trigger a retry */
  retryOnStatuses: number[];
}

/** Per-request configuration overrides */
export interface RequestConfig {
  /** Request timeout in milliseconds */
  timeout?: number;
  /** Retry policy for this specific request */
  retry?: Partial<RetryConfig> | false;
  /** Additional headers to merge */
  headers?: Record<string, string>;
}

/** Signature provider for ed25519-based request signing */
export interface SignatureAuth {
  /** Hex-encoded ed25519 public key */
  publicKey: string;
  /** Signs the request body bytes and returns a hex-encoded ed25519 signature */
  sign: (body: Uint8Array) => Promise<string>;
}

export class ApiError extends Error {
  constructor(
    message: string,
    public statusCode?: number,
    public response?: any
  ) {
    super(message);
    this.name = "ApiError";
  }
}

export class ApiClient {
  private baseUrl: string;
  private getAuthToken: () => string | null;
  private signatureAuth: SignatureAuth | null = null;
  private defaultTimeout: number;
  private defaultRetry: RetryConfig;

  constructor(
    baseUrl: string = API_BASE_URL,
    getAuthToken: () => string | null = () => {
      if (typeof window !== "undefined") {
        return localStorage.getItem("auth_token");
      }
      return null;
    }
  ) {
    this.baseUrl = baseUrl;
    this.getAuthToken = getAuthToken;
    this.defaultTimeout = DEFAULT_TIMEOUT_MS;
    this.defaultRetry = { ...DEFAULT_RETRY_CONFIG };
  }

  /**
   * Configure signature-based authentication using an ed25519 key pair.
   * When set, X-Public-Key and X-Signature headers are attached to every request.
   */
  setSignatureAuth(auth: SignatureAuth | null): void {
    this.signatureAuth = auth;
  }

  /**
   * Configure the default request timeout.
   */
  setDefaultTimeout(ms: number): void {
    this.defaultTimeout = ms;
  }

  /**
   * Configure the default retry policy.
   */
  setDefaultRetry(config: Partial<RetryConfig>): void {
    this.defaultRetry = { ...this.defaultRetry, ...config };
  }

  /**
   * Compute the delay for a given retry attempt using exponential backoff
   * with full jitter.
   */
  private computeRetryDelay(attempt: number, config: RetryConfig): number {
    const exponentialDelay = config.baseDelayMs * Math.pow(2, attempt);
    const cappedDelay = Math.min(exponentialDelay, config.maxDelayMs);
    // Full jitter: random value between 0 and cappedDelay
    return Math.random() * cappedDelay;
  }

  /**
   * Determine whether a response should be retried based on its status code.
   */
  private shouldRetry(
    statusCode: number | undefined,
    retryOnStatuses: number[]
  ): boolean {
    if (statusCode === undefined) return true; // network error → retry
    return retryOnStatuses.includes(statusCode);
  }

  /**
   * Sleep for the given number of milliseconds.
   */
  private delay(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  /**
   * Core request method with timeout, retry, and dual auth support.
   */
  private async request<T>(
    endpoint: string,
    options: RequestInit = {},
    config: RequestConfig = {}
  ): Promise<T> {
    const retryConfig: RetryConfig | false =
      config.retry === false
        ? false
        : { ...this.defaultRetry, ...(config.retry || {}) };

    const timeoutMs = config.timeout ?? this.defaultTimeout;
    const maxAttempts = retryConfig ? retryConfig.maxRetries + 1 : 1;

    let lastError: unknown;

    for (let attempt = 0; attempt < maxAttempts; attempt++) {
      // Apply backoff delay before retries (skip first attempt)
      if (attempt > 0 && retryConfig) {
        const waitMs = this.computeRetryDelay(attempt - 1, retryConfig);
        await this.delay(waitMs);
      }

      let timeoutId: ReturnType<typeof setTimeout> | undefined;

      try {
        // Build headers
        const headers: Record<string, string> = {
          "Content-Type": "application/json",
          ...((options.headers as Record<string, string>) || {}),
          ...(config.headers || {}),
        };

        // Attach Bearer token for JWT-based admin auth
        const token = this.getAuthToken();
        if (token) {
          headers["Authorization"] = `Bearer ${token}`;
        }

        // Attach signature headers for ed25519 user auth.
        // NOTE: signature auth currently only supports JSON string bodies.
        // Non-string bodies (FormData, Blob, etc.) will not be signed.
        if (this.signatureAuth) {
          headers["X-Public-Key"] = this.signatureAuth.publicKey;

          // Sign the request body if present
          const bodyStr =
            typeof options.body === "string" ? options.body : "";
          if (bodyStr) {
            const bodyBytes = new TextEncoder().encode(bodyStr);
            const signature = await this.signatureAuth.sign(bodyBytes);
            headers["X-Signature"] = signature;
          }
        }

        const mergedInit: RequestInit = {
          ...options,
          headers,
        };

        // Timeout via Promise.race avoids AbortSignal compatibility issues
        // in jsdom test environments where jsdom's AbortSignal class differs
        // from Node's native AbortSignal used by MSW's fetch interception.
        const response = await Promise.race([
          fetch(`${this.baseUrl}${endpoint}`, mergedInit),
          new Promise<never>((_, reject) => {
            timeoutId = setTimeout(
              () => reject(new ApiError(`Request timed out after ${timeoutMs}ms`, 408)),
              timeoutMs
            );
          }),
        ]);

        if (timeoutId !== undefined) {
          clearTimeout(timeoutId);
          timeoutId = undefined;
        }

        // If response is retriable and we have attempts left, retry
        if (
          retryConfig &&
          attempt < maxAttempts - 1 &&
          this.shouldRetry(response.status, retryConfig.retryOnStatuses)
        ) {
          continue;
        }

        // Handle non-JSON responses
        const contentType = response.headers.get("content-type");
        if (!contentType || !contentType.includes("application/json")) {
          if (!response.ok) {
            throw new ApiError(
              `Request failed with status ${response.status}`,
              response.status
            );
          }
          return {} as T;
        }

        const data = await response.json();

        if (!response.ok) {
          throw new ApiError(
            data.error ||
              data.message ||
              `Request failed with status ${response.status}`,
            response.status,
            data
          );
        }

        return data;
      } catch (error) {
        // Always clean up the timeout to prevent leaks
        if (timeoutId !== undefined) {
          clearTimeout(timeoutId);
          timeoutId = undefined;
        }

        // Re-throw ApiErrors that shouldn't be retried
        if (error instanceof ApiError) {
          if (
            retryConfig &&
            attempt < maxAttempts - 1 &&
            this.shouldRetry(error.statusCode, retryConfig.retryOnStatuses)
          ) {
            lastError = error;
            continue;
          }
          throw error;
        }

        // Network/abort errors — retry if possible
        if (
          retryConfig &&
          attempt < maxAttempts - 1
        ) {
          lastError = error;
          continue;
        }

        if (error instanceof Error) {
          throw new ApiError(error.message);
        }
        throw new ApiError("An unknown error occurred");
      }
    }

    // Exhausted all retries
    if (lastError instanceof ApiError) {
      throw lastError;
    }
    if (lastError instanceof Error) {
      throw new ApiError(lastError.message);
    }
    throw new ApiError("Request failed after all retry attempts");
  }

  /**
   * GET request
   */
  async get<T>(
    endpoint: string,
    config?: RequestConfig
  ): Promise<T> {
    return this.request<T>(endpoint, { method: "GET" }, config);
  }

  /**
   * POST request
   */
  async post<T>(
    endpoint: string,
    body?: any,
    config?: RequestConfig
  ): Promise<T> {
    return this.request<T>(
      endpoint,
      {
        method: "POST",
        body: body ? JSON.stringify(body) : undefined,
      },
      config
    );
  }

  /**
   * PUT request
   */
  async put<T>(
    endpoint: string,
    body?: any,
    config?: RequestConfig
  ): Promise<T> {
    return this.request<T>(
      endpoint,
      {
        method: "PUT",
        body: body ? JSON.stringify(body) : undefined,
      },
      config
    );
  }

  /**
   * DELETE request
   */
  async delete<T>(
    endpoint: string,
    config?: RequestConfig
  ): Promise<T> {
    return this.request<T>(endpoint, { method: "DELETE" }, config);
  }

  /**
   * Get the configured base URL.
   */
  getBaseUrl(): string {
    return this.baseUrl;
  }

  /**
   * Set authentication token
   */
  setAuthToken(token: string) {
    if (typeof window !== "undefined") {
      localStorage.setItem("auth_token", token);
    }
  }

  /**
   * Clear authentication token
   */
  clearAuthToken() {
    if (typeof window !== "undefined") {
      localStorage.removeItem("auth_token");
    }
  }

  /**
   * Check if user is authenticated
   */
  isAuthenticated(): boolean {
    return !!this.getAuthToken();
  }
}

// Create singleton instance
export const apiClient = new ApiClient();

export default apiClient;
