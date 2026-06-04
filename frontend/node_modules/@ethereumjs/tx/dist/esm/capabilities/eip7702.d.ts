import type { EIP7702CompatibleTx } from '../types.ts';
/**
 * The amount of gas paid for the data in this tx
 */
export declare function getDataGas(tx: EIP7702CompatibleTx): bigint;
/**
 * Checks if the authorization list is valid. Throws if invalid.
 * @param tx - Transaction whose authorization list should be validated
 */
export declare function verifyAuthorizationList(tx: EIP7702CompatibleTx): void;
//# sourceMappingURL=eip7702.d.ts.map