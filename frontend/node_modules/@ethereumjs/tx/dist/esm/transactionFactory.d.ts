import { TransactionType } from './types.ts';
import type { EthersProvider } from '@ethereumjs/util';
import type { Transaction, TxData, TxOptions, TypedTxData } from './types.ts';
/**
 * Create a transaction from a `txData` object
 *
 * @param txData - The transaction data. The `type` field will determine which transaction type is returned (if undefined, creates a legacy transaction)
 * @param txOptions - Options to pass on to the constructor of the transaction
 */
export declare function createTx<T extends TransactionType>(txData: TypedTxData, txOptions?: TxOptions): Transaction[T];
/**
 * This method tries to decode serialized data.
 *
 * @param data - The data Uint8Array
 * @param txOptions - The transaction options
 */
export declare function createTxFromRLP<T extends TransactionType>(data: Uint8Array, txOptions?: TxOptions): Transaction[T];
/**
 * When decoding a BlockBody, in the transactions field, a field is either:
 * A Uint8Array (a TypedTransaction - encoded as TransactionType || rlp(TransactionPayload))
 * A Uint8Array[] (Legacy Transaction)
 * This method returns the right transaction.
 *
 * @param data - A Uint8Array or Uint8Array[]
 * @param txOptions - The transaction options
 */
export declare function createTxFromBlockBodyData(data: Uint8Array | Uint8Array[], txOptions?: TxOptions): import("./index.ts").LegacyTx | import("./index.ts").AccessList2930Tx | import("./index.ts").FeeMarket1559Tx | import("./index.ts").Blob4844Tx | import("./index.ts").EOACode7702Tx;
/**
 * Method to decode data retrieved from RPC, such as `eth_getTransactionByHash`
 * Note that this normalizes some of the parameters
 * @param txData The RPC-encoded data
 * @param txOptions The transaction options
 * @returns A promise that resolves with the instantiated transaction
 */
export declare function createTxFromRPC<T extends TransactionType>(txData: TxData[T], txOptions?: TxOptions): Promise<Transaction[T]>;
/**
 *  Method to retrieve a transaction from the provider
 * @param provider - a url string for a JSON-RPC provider or an Ethers JSONRPCProvider object
 * @param txHash - Transaction hash
 * @param txOptions - The transaction options
 * @returns the transaction specified by `txHash`
 */
export declare function createTxFromJSONRPCProvider(provider: string | EthersProvider, txHash: string, txOptions?: TxOptions): Promise<import("./index.ts").LegacyTx | import("./index.ts").AccessList2930Tx | import("./index.ts").FeeMarket1559Tx | import("./index.ts").Blob4844Tx | import("./index.ts").EOACode7702Tx>;
//# sourceMappingURL=transactionFactory.d.ts.map