import type { CoinSelectRequest, ComposeChangeAddress, ComposeInput, ComposeOutput, ComposeRequest, ComposeResultError } from '../types';
type Request = ComposeRequest<ComposeInput, ComposeOutput, ComposeChangeAddress>;
export declare function validateAndParseRequest(request: Request): CoinSelectRequest | ComposeResultError;
export {};
//# sourceMappingURL=request.d.ts.map