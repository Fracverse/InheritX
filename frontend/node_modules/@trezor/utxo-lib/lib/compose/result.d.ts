import { CoinSelectRequest, CoinSelectResult, ComposeChangeAddress, ComposeInput, ComposeOutput, ComposeRequest, ComposeResult, ComposeResultError } from '../types';
export declare function getErrorResult(error: unknown): ComposeResultError;
export declare function getResult<Input extends ComposeInput, Output extends ComposeOutput, Change extends ComposeChangeAddress>(request: ComposeRequest<Input, Output, Change>, { sendMaxOutputIndex }: CoinSelectRequest, result: CoinSelectResult): ComposeResult<Input, Output, Change>;
//# sourceMappingURL=result.d.ts.map