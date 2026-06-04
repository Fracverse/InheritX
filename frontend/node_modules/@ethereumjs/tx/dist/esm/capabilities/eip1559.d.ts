import type { EIP1559CompatibleTx } from '../types.ts';
/**
 * Calculates the total upfront wei cost required for an EIP-1559 compatible transaction.
 * @param tx - Transaction whose costs should be evaluated
 * @param baseFee - Base fee from the parent block used to determine inclusion cost
 * @returns Total wei that must be available to cover gas and value
 */
export declare function getUpfrontCost(tx: EIP1559CompatibleTx, baseFee: bigint): bigint;
/**
 * Determines the effective priority fee that can be paid to the block proposer.
 * @param tx - Transaction whose priority fee is being computed
 * @param baseFee - Base fee from the parent block; must be payable by the tx
 * @returns The lesser of `maxPriorityFeePerGas` and the remaining fee after base fee deduction
 * @throws EthereumJSErrorWithoutCode if the base fee exceeds `maxFeePerGas`
 */
export declare function getEffectivePriorityFee(tx: EIP1559CompatibleTx, baseFee: bigint | undefined): bigint;
//# sourceMappingURL=eip1559.d.ts.map