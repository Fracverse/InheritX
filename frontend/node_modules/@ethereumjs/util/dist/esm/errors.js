import { DEFAULT_ERROR_CODE, EthereumJSError, EthereumJSErrorWithoutCode, } from '@ethereumjs/rlp';
export { DEFAULT_ERROR_CODE, EthereumJSError, EthereumJSErrorWithoutCode, };
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