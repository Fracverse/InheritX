import type { NextConfig } from "next";
import createNextIntlPlugin from "next-intl/plugin";
import withBundleAnalyzer from "@next/bundle-analyzer";

const withNextIntl = createNextIntlPlugin("./i18n/request.ts");

const withBundleAnalyzerConfig = withBundleAnalyzer({
  enabled: process.env.ANALYZE === "true",
});

/**
 * Security headers are primarily applied by middleware.ts on every request.
 * The headers() config below acts as a complementary layer — it covers
 * static-file responses and CDN-cached pages that bypass middleware.
 *
 * Issue #417 — Add Security Headers Middleware
 */
const securityHeaders = [
  // Prevent MIME-type sniffing
  { key: "X-Content-Type-Options", value: "nosniff" },
  // Clickjacking protection (legacy browsers; CSP frame-ancestors handles modern ones)
  { key: "X-Frame-Options", value: process.env.X_FRAME_OPTIONS ?? "DENY" },
  // Legacy XSS filter
  { key: "X-XSS-Protection", value: "1; mode=block" },
  // Referrer information control
  {
    key: "Referrer-Policy",
    value: process.env.REFERRER_POLICY ?? "strict-origin-when-cross-origin",
  },
  // Restrict browser feature access
  {
    key: "Permissions-Policy",
    value:
      process.env.PERMISSIONS_POLICY ??
      "camera=(), microphone=(), geolocation=(), payment=(), usb=(), interest-cohort=()",
  },
];

const nextConfig: NextConfig = {
  // Only transpile allbridge if it is installed (optional dep).
  // Next.js silently ignores transpilePackages entries for missing modules.
  transpilePackages: ["@allbridge/bridge-core-sdk"],
  images: {
    remotePatterns: [
      {
        protocol: "https",
        hostname: "images.unsplash.com",
      },
      {
        protocol: "https",
        hostname: "plus.unsplash.com",
      },
    ],
  },

  compiler: {
    removeConsole: process.env.NODE_ENV === "production",
  },

  async headers() {
    return [
      {
        // Apply to all routes
        source: "/(.*)",
        headers: securityHeaders,
      },
    ];
  },
};

// Compile time dynamic require check for optional PWA dependency
let withPWA = (config: any) => config;
if (process.env.NODE_ENV !== "development") {
  try {
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const withPWAInit = require("@ducanh2912/next-pwa");
    const pwaWrapper = withPWAInit.default || withPWAInit;
    withPWA = pwaWrapper({
      dest: "public",
      register: true,
    });
  } catch (e) {
    // optional dependency not installed
  }
}

const resolvedConfig = withPWA(withBundleAnalyzerConfig(withNextIntl(nextConfig)));
export default resolvedConfig;
