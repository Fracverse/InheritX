import { CompilableTransactionMessage, CompiledTransactionMessage, GetFeeForMessageApi, GetRecentPrioritizationFeesApi, Rpc, SignaturesMap, SimulateTransactionApi } from '@solana/kit';
export declare const getBaseFee: (api: Rpc<GetFeeForMessageApi>, message: CompiledTransactionMessage) => Promise<import("@solana/kit").Lamports>;
export declare const getPriorityFee: (api: Rpc<GetRecentPrioritizationFeesApi & SimulateTransactionApi>, decompiledMessage: CompilableTransactionMessage, compiledMessage: CompiledTransactionMessage, signatures: SignaturesMap) => Promise<{
    computeUnitPrice: string;
    computeUnitLimit: string;
    fee: string;
}>;
//# sourceMappingURL=fee.d.ts.map