import createMiddleware from "next-intl/middleware";
import { locales, defaultLocale } from "./i18n/config";

/**
 * next-intl routing middleware
 *
 * Responsibilities:
 * - Detects browser language from Accept-Language header
 * - Redirects to the matched locale prefix (e.g. /fr/...) if supported
 * - Falls back to defaultLocale ("en") if browser language is unsupported
 * - Persists the chosen locale in a cookie (NEXT_LOCALE) so subsequent
 *   visits skip detection — this also satisfies the localStorage/cookie
 *   persistence requirement for language selection
 */
export default createMiddleware({
  locales,
  defaultLocale,
  // "always" means every URL has an explicit locale prefix (/en, /fr, /es, /pt)
  // "as-needed" omits the prefix for the default locale — better UX for EN users
  localePrefix: "as-needed",
  // Persist language choice in a cookie named NEXT_LOCALE
  localeDetection: true,
});

export const config = {
  // Match all paths except Next.js internals and static files
  matcher: [
    "/((?!_next|_vercel|.*\\..*).*)",
  ],
};
