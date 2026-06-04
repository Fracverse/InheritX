import { type CompilableTransactionMessage } from '@solana/kit';
import type { TokenAccount } from '@trezor/blockchain-link-types';
import type { TokenProgramName } from '@trezor/blockchain-link-utils/lib/solana';
import { BigNumber } from '@trezor/utils/lib/bigNumber';
import { Blockchain } from '../../backend/Blockchain';
export declare const SOLANA_BASE_FEE = 5000;
export declare const getLamportsFromSol: (amountInSol: string) => bigint;
type PriorityFees = {
    computeUnitPrice: string;
    computeUnitLimit: string;
};
export declare const dummyPriorityFeesForFeeEstimation: PriorityFees;
export declare function createTransactionShim(message: CompilableTransactionMessage): Promise<{
    addSignature(signerPubKey: string, signatureHex: string): void;
    serializeMessage(): string;
    serialize(): string;
}>;
export declare function createTransactionShimFromHex(rawTx: string): Promise<{
    addSignature(signerPubKey: string, signatureHex: string): void;
    serializeMessage(): string;
    serialize(): string;
}>;
export declare const buildTransferTransaction: (fromAddress: string, toAddress: string, amountInSol: string, blockhash: string, lastValidBlockHeight: number, priorityFees: PriorityFees) => Promise<{
    addSignature(signerPubKey: string, signatureHex: string): void;
    serializeMessage(): string;
    serialize(): string;
}>;
export declare const buildTokenTransferInstruction: (from: string, to: string, owner: string, amount: BigNumber, mint: string, decimals: number, tokenProgramName: TokenProgramName) => Promise<import("@solana-program/token").TransferCheckedInstruction<import("@solana/kit").Address<"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA">, string, string, string, string, []> | import("@solana-program/token-2022").TransferCheckedInstruction<import("@solana/kit").Address<"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA">, string, string, string, string, []>>;
export declare const getAssociatedTokenAccountAddress: (baseAddress: string, tokenMintAddress: string, tokenProgramName: TokenProgramName) => Promise<import("@solana/kit").Address<string>>;
export declare const buildCreateAssociatedTokenAccountInstruction: (funderAddress: string, newOwnerAddress: string, tokenMintAddress: string, tokenProgramName: TokenProgramName) => Promise<readonly [import("@solana-program/token").CreateAssociatedTokenInstruction<import("@solana/kit").Address<"ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL">, string, string, string, string, string, string, []>, import("@solana/kit").Address<string>]>;
type TokenTransferTxWithDestinationAddress = {
    transaction: {
        addSignature(signerPubKey: string, signatureHex: string): void;
        serializeMessage(): string;
        serialize(): string;
    };
    destinationAddress: string;
    tokenAccountInfo?: {
        baseAddress: string;
        tokenProgram: string;
        tokenMint: string;
        tokenAccount: string;
    };
};
export declare const getMinimumRequiredTokenAccountsForTransfer: (tokenAccounts: TokenAccount[], requiredAmount: string) => TokenAccount[];
export declare const buildTokenTransferTransaction: (fromAddress: string, toAddress: string, toAddressOwner: string, tokenMint: string, tokenUiAmount: string, tokenDecimals: number, fromTokenAccounts: TokenAccount[], toTokenAccount: TokenAccount | undefined, blockhash: string, lastValidBlockHeight: number, priorityFees: PriorityFees, tokenProgramName: TokenProgramName) => Promise<TokenTransferTxWithDestinationAddress>;
export declare const fetchAccountOwnerAndTokenInfoForAddress: (blockchain: Blockchain, address: string, mint: string, tokenProgram: TokenProgramName) => Promise<readonly [string | undefined, TokenAccount | undefined]>;
export {};
//# sourceMappingURL=solanaUtils.d.ts.map