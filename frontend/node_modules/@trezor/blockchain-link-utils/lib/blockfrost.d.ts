import type { VinVout } from '@trezor/blockchain-link-types/lib/blockbook';
import type { AssetBalance, BlockfrostAccountInfo, BlockfrostTransaction, BlockfrostUtxos, ParseAssetResult } from '@trezor/blockchain-link-types/lib/blockfrost';
import type { AccountAddresses, AccountInfo, TokenInfo, TokenTransfer, Transaction, TransferType, Utxo } from '@trezor/blockchain-link-types/lib/common';
export declare const transformUtxos: (utxos: BlockfrostUtxos[]) => Utxo[];
export declare const parseAsset: (hex: string) => ParseAssetResult;
export declare const transformToken: (token: AssetBalance) => {
    name: string;
    contract: string;
    symbol: string;
    decimals: number;
    fingerprint: string;
    policyId: string;
};
export declare const transformTokenInfo: (tokens: BlockfrostAccountInfo["tokens"]) => TokenInfo[] | undefined;
export declare const transformInputOutput: (data: BlockfrostTransaction["txUtxos"]["inputs"] | BlockfrostTransaction["txUtxos"]["outputs"], asset?: string) => VinVout[];
export declare const filterTokenTransfers: (accountAddress: AccountAddresses, tx: BlockfrostTransaction, type: TransferType) => TokenTransfer[];
export declare const transformTransaction: (blockfrostTxData: BlockfrostTransaction | Pick<BlockfrostTransaction, "txData">, addressesOrDescriptor?: AccountAddresses | string) => Transaction;
export declare const transformAccountInfo: (info: BlockfrostAccountInfo) => AccountInfo;
//# sourceMappingURL=blockfrost.d.ts.map