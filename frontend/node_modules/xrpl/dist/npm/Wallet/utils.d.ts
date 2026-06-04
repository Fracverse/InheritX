import BigNumber from 'bignumber.js';
import { Transaction } from '../models';
export declare function compareSigners<T extends {
    Account: string;
}>(left: T, right: T): number;
export declare const NUM_BITS_IN_HEX = 16;
export declare function addressToBigNumber(address: string): BigNumber;
export declare function getDecodedTransaction(txOrBlob: Transaction | string): Transaction;
export declare function computeSignature(tx: Transaction, privateKey: string, signAs?: string): string;
//# sourceMappingURL=utils.d.ts.map