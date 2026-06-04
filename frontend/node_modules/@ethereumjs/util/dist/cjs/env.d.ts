/**
 * Safely checks if debug logging is enabled for the given namespace string.
 * Works in Node.js, browser main thread, web workers, and service workers.
 *
 * Uses `globalThis.process` to avoid ReferenceError in environments where
 * `process` is not a declared global (e.g., web workers, service workers).
 */
export declare const isDebugEnabled: (namespace: string) => boolean;
//# sourceMappingURL=env.d.ts.map