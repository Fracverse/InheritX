"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isDebugEnabled = void 0;
/**
 * Safely checks if debug logging is enabled for the given namespace string.
 * Works in Node.js, browser main thread, web workers, and service workers.
 *
 * Uses `globalThis.process` to avoid ReferenceError in environments where
 * `process` is not a declared global (e.g., web workers, service workers).
 */
const isDebugEnabled = (namespace) => globalThis.process?.env?.DEBUG?.includes(namespace) ?? false;
exports.isDebugEnabled = isDebugEnabled;
//# sourceMappingURL=env.js.map