import type { EIP2930CompatibleTx } from '../types.ts';
/**
 * The amount of gas paid for the data in this tx
 */
export declare function getDataGas(tx: EIP2930CompatibleTx): bigint;
/**
 * Verifies an access list. Throws if invalid.
 * @param tx - Transaction whose access list should be validated
 */
export declare function verifyAccessList(tx: EIP2930CompatibleTx): void;
//# sourceMappingURL=eip2930.d.ts.map