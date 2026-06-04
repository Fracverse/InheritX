"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getUpfrontCost = getUpfrontCost;
exports.getEffectivePriorityFee = getEffectivePriorityFee;
const util_1 = require("@ethereumjs/util");
/**
 * Calculates the total upfront wei cost required for an EIP-1559 compatible transaction.
 * @param tx - Transaction whose costs should be evaluated
 * @param baseFee - Base fee from the parent block used to determine inclusion cost
 * @returns Total wei that must be available to cover gas and value
 */
function getUpfrontCost(tx, baseFee) {
    const prio = tx.maxPriorityFeePerGas;
    const maxBase = tx.maxFeePerGas - baseFee;
    const inclusionFeePerGas = prio < maxBase ? prio : maxBase;
    const gasPrice = inclusionFeePerGas + baseFee;
    return tx.gasLimit * gasPrice + tx.value;
}
/**
 * Determines the effective priority fee that can be paid to the block proposer.
 * @param tx - Transaction whose priority fee is being computed
 * @param baseFee - Base fee from the parent block; must be payable by the tx
 * @returns The lesser of `maxPriorityFeePerGas` and the remaining fee after base fee deduction
 * @throws EthereumJSErrorWithoutCode if the base fee exceeds `maxFeePerGas`
 */
function getEffectivePriorityFee(tx, baseFee) {
    if (baseFee === undefined || baseFee > tx.maxFeePerGas) {
        throw (0, util_1.EthereumJSErrorWithoutCode)('Tx cannot pay baseFee');
    }
    // The remaining fee for the coinbase, which can take up to this value, capped at `maxPriorityFeePerGas`
    const remainingFee = tx.maxFeePerGas - baseFee;
    return tx.maxPriorityFeePerGas < remainingFee ? tx.maxPriorityFeePerGas : remainingFee;
}
//# sourceMappingURL=eip1559.js.map