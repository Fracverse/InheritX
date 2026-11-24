import { ApiError } from "@/app/lib/api/client";

/**
 * Turns a failed admin request into a message that says what failed and what
 * the admin can do about it.
 *
 * @param action Short description of what was attempted, e.g. "Suspending GCKF4X...3X9P".
 */
export function describeAdminError(error: unknown, action: string): string {
  if (error instanceof ApiError) {
    switch (error.statusCode) {
      case 401:
        return `${action} failed: your admin session is missing or has expired. Sign in again, then retry.`;
      case 403:
        return `${action} failed: your account is not permitted to do this.`;
      case 404:
        return `${action} failed: the server could not find this record or endpoint. Refresh the page and try again.`;
      case 408:
      case undefined:
        return `${action} failed: could not reach the server. Check your connection and try again.`;
      default:
        return `${action} failed: ${error.message}`;
    }
  }
  return `${action} failed: ${error instanceof Error ? error.message : "unknown error"}.`;
}
