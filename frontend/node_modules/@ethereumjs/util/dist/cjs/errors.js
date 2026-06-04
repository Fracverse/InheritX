"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.EthereumJSErrorWithoutCode = exports.EthereumJSError = exports.DEFAULT_ERROR_CODE = void 0;
const rlp_1 = require("@ethereumjs/rlp");
Object.defineProperty(exports, "DEFAULT_ERROR_CODE", { enumerable: true, get: function () { return rlp_1.DEFAULT_ERROR_CODE; } });
Object.defineProperty(exports, "EthereumJSError", { enumerable: true, get: function () { return rlp_1.EthereumJSError; } });
Object.defineProperty(exports, "EthereumJSErrorWithoutCode", { enumerable: true, get: function () { return rlp_1.EthereumJSErrorWithoutCode; } });
// Below here: specific monorepo-wide errors (examples and commented out)
/*export enum UsageErrorType {
  UNSUPPORTED_FEATURE = 'unsupported feature',
}*

/**
 * Error along API Usage
 *
 * Use directly or in a subclassed context for error comparison (`e instanceof UsageError`)
 */
//export class UsageError extends EthereumJSError<{ code: UsageErrorType }> {}
//# sourceMappingURL=errors.js.map