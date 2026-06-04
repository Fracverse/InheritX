import { NextRequest, NextResponse } from "next/server";

/**
 * Security Headers Middleware — Issue #417
 *
 * Applies the following headers to every response:
 *  - Strict-Transport-Security (HSTS)
 *  - Content-Security-Policy (CSP)
 *  - X-Frame-Options
 *  - X-Content-Type-Options
 *  - Referrer-Policy
 *  - Permissions-Policy
 *  - X-XSS-Protection (legacy browsers)
 *
 * All values can be overridden via environment variables so that
 * staging / production environments can tighten or relax policies
 * without a code change.
 */

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/**
 * Build the Content-Security-Policy header value.
 *
 * The default policy is intentionally permissive enough for the current
 * InheritX frontend (Next.js, Stellar wallet kit, Framer Motion, Google
 * Fonts, Unsplash images) while still blocking the most common injection
 * vectors.
 *
 * Override the entire policy by setting CSP_HEADER in the environment.
 */
function buildCSP(nonce: string): string {
    // Allow an env-level override for the whole policy (e.g. in production).
    if (process.env.CSP_HEADER) {
        return process.env.CSP_HEADER;
    }

    const isDev = process.env.NODE_ENV === "development";

    // In development Next.js injects inline scripts/styles for HMR, so we
    // must allow 'unsafe-inline' and 'unsafe-eval'. In production we use the
    // nonce-based approach instead.
    const scriptSrc = isDev
        ? `'self' 'unsafe-inline' 'unsafe-eval'`
        : `'self' 'nonce-${nonce}' 'strict-dynamic'`;

    const styleSrc = isDev
        ? `'self' 'unsafe-inline' https://fonts.googleapis.com`
        : `'self' 'unsafe-inline' https://fonts.googleapis.com`;

    // Trusted external connect targets (Stellar, Horizon, Soroban, etc.)
    const connectSrc = [
        "'self'",
        // Stellar / Soroban RPC
        "https://horizon-testnet.stellar.org",
        "https://horizon.stellar.org",
        "https://soroban-testnet.stellar.org",
        "https://soroban.stellar.org",
        // Wallet extensions communicate via postMessage, not fetch, but some
        // use background service workers that open connections.
        "wss:",
        // Allow any additional origins configured at deploy time.
        process.env.CSP_CONNECT_SRC_EXTRA ?? "",
    ]
        .filter(Boolean)
        .join(" ");

    const imgSrc = [
        "'self'",
        "data:",
        "blob:",
        "https://images.unsplash.com",
        "https://plus.unsplash.com",
        // Wallet icons are loaded from the kit's own CDN / data URIs.
        "https:",
        process.env.CSP_IMG_SRC_EXTRA ?? "",
    ]
        .filter(Boolean)
        .join(" ");

    const fontSrc = [
        "'self'",
        "https://fonts.gstatic.com",
        process.env.CSP_FONT_SRC_EXTRA ?? "",
    ]
        .filter(Boolean)
        .join(" ");

    const frameSrc = [
        // Albedo wallet opens in an iframe.
        "https://albedo.link",
        process.env.CSP_FRAME_SRC_EXTRA ?? "",
    ]
        .filter(Boolean)
        .join(" ");

    const workerSrc = ["'self'", "blob:"].join(" ");

    const directives: string[] = [
        `default-src 'self'`,
        `script-src ${scriptSrc}`,
        `style-src ${styleSrc}`,
        `img-src ${imgSrc}`,
        `font-src ${fontSrc}`,
        `connect-src ${connectSrc}`,
        `frame-src ${frameSrc}`,
        `worker-src ${workerSrc}`,
        `object-src 'none'`,
        `base-uri 'self'`,
        `form-action 'self'`,
        `frame-ancestors 'none'`,
        // Upgrade insecure requests in production only.
        ...(isDev ? [] : ["upgrade-insecure-requests"]),
    ];

    return directives.join("; ");
}

/**
 * Generate a cryptographically random nonce for use in CSP script-src.
 * Falls back to a timestamp-based value in environments where
 * crypto.getRandomValues is unavailable (e.g. some edge runtimes).
 */
function generateNonce(): string {
    try {
        const bytes = new Uint8Array(16);
        crypto.getRandomValues(bytes);
        return Buffer.from(bytes).toString("base64");
    } catch {
        return Buffer.from(Date.now().toString()).toString("base64");
    }
}

// ---------------------------------------------------------------------------
// Middleware
// ---------------------------------------------------------------------------

export function middleware(request: NextRequest): NextResponse {
    const response = NextResponse.next();

    const nonce = generateNonce();

    // ── 1. Strict-Transport-Security (HSTS) ──────────────────────────────────
    // Tells browsers to only use HTTPS for this domain.
    // max-age is configurable; default is 1 year (recommended minimum).
    // includeSubDomains and preload can be enabled via env.
    const hstsMaxAge = process.env.HSTS_MAX_AGE ?? "31536000"; // 1 year
    const hstsIncludeSubDomains =
        process.env.HSTS_INCLUDE_SUBDOMAINS !== "false"; // default true
    const hstsPreload = process.env.HSTS_PRELOAD === "true"; // default false

    let hstsValue = `max-age=${hstsMaxAge}`;
    if (hstsIncludeSubDomains) hstsValue += "; includeSubDomains";
    if (hstsPreload) hstsValue += "; preload";

    // Only send HSTS over HTTPS (not on localhost).
    if (request.nextUrl.protocol === "https:") {
        response.headers.set("Strict-Transport-Security", hstsValue);
    }

    // ── 2. Content-Security-Policy ───────────────────────────────────────────
    const csp = buildCSP(nonce);
    response.headers.set("Content-Security-Policy", csp);

    // Expose the nonce to the page via a request header so that
    // Server Components can read it and inject it into <script> tags.
    response.headers.set("x-nonce", nonce);

    // ── 3. X-Frame-Options ───────────────────────────────────────────────────
    // Prevents the page from being embedded in an iframe (clickjacking).
    // CSP frame-ancestors 'none' is the modern equivalent, but we keep this
    // for older browsers that don't support CSP.
    const xFrameOptions = process.env.X_FRAME_OPTIONS ?? "DENY";
    response.headers.set("X-Frame-Options", xFrameOptions);

    // ── 4. X-Content-Type-Options ────────────────────────────────────────────
    // Prevents browsers from MIME-sniffing a response away from the declared
    // content type (e.g. serving a JS file as text/html).
    response.headers.set("X-Content-Type-Options", "nosniff");

    // ── 5. Referrer-Policy ───────────────────────────────────────────────────
    // Controls how much referrer information is sent with requests.
    // strict-origin-when-cross-origin is the recommended modern default.
    const referrerPolicy =
        process.env.REFERRER_POLICY ?? "strict-origin-when-cross-origin";
    response.headers.set("Referrer-Policy", referrerPolicy);

    // ── 6. Permissions-Policy ────────────────────────────────────────────────
    // Restricts access to browser features. Deny everything not needed by
    // the app; camera/microphone are not used by InheritX.
    const permissionsPolicy =
        process.env.PERMISSIONS_POLICY ??
        "camera=(), microphone=(), geolocation=(), payment=(), usb=(), interest-cohort=()";
    response.headers.set("Permissions-Policy", permissionsPolicy);

    // ── 7. X-XSS-Protection (legacy) ─────────────────────────────────────────
    // Enables the XSS filter built into older browsers (IE, legacy Chrome/Safari).
    // Modern browsers rely on CSP instead, but this doesn't hurt.
    response.headers.set("X-XSS-Protection", "1; mode=block");

    return response;
}

// ---------------------------------------------------------------------------
// Matcher — apply to all routes except Next.js internals and static assets
// ---------------------------------------------------------------------------

export const config = {
    matcher: [
        /*
         * Match all request paths EXCEPT:
         *  - _next/static  (static files)
         *  - _next/image   (image optimisation)
         *  - favicon.ico
         *  - public folder assets (png, jpg, svg, etc.)
         */
        "/((?!_next/static|_next/image|favicon.ico|.*\\.(?:png|jpg|jpeg|gif|svg|ico|webp|woff|woff2|ttf|otf|eot)$).*)",
    ],
};
