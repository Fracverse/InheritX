"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateLoanBrokerSet = void 0;
const bignumber_js_1 = __importDefault(require("bignumber.js"));
const errors_1 = require("../../errors");
const common_1 = require("./common");
const MAX_DATA_LENGTH = 512;
const MAX_MANAGEMENT_FEE_RATE = 10000;
const MAX_COVER_RATE_MINIMUM = 100000;
const MAX_COVER_RATE_LIQUIDATION = 100000;
function validateLoanBrokerSet(tx) {
    var _a, _b;
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateRequiredField)(tx, 'VaultID', common_1.isString);
    (0, common_1.validateOptionalField)(tx, 'LoanBrokerID', common_1.isString);
    (0, common_1.validateOptionalField)(tx, 'Data', common_1.isString);
    (0, common_1.validateOptionalField)(tx, 'ManagementFeeRate', common_1.isNumber);
    (0, common_1.validateOptionalField)(tx, 'DebtMaximum', common_1.isXRPLNumber);
    (0, common_1.validateOptionalField)(tx, 'CoverRateMinimum', common_1.isNumber);
    (0, common_1.validateOptionalField)(tx, 'CoverRateLiquidation', common_1.isNumber);
    if (!(0, common_1.isLedgerEntryId)(tx.VaultID)) {
        throw new errors_1.ValidationError(`LoanBrokerSet: VaultID must be 64 characters hexadecimal string`);
    }
    if (tx.LoanBrokerID != null && !(0, common_1.isLedgerEntryId)(tx.LoanBrokerID)) {
        throw new errors_1.ValidationError(`LoanBrokerSet: LoanBrokerID must be 64 characters hexadecimal string`);
    }
    if (tx.Data != null && !(0, common_1.validateHexMetadata)(tx.Data, MAX_DATA_LENGTH)) {
        throw new errors_1.ValidationError(`LoanBrokerSet: Data must be a valid non-empty hex string up to ${MAX_DATA_LENGTH} characters`);
    }
    if (tx.ManagementFeeRate != null &&
        (tx.ManagementFeeRate < 0 || tx.ManagementFeeRate > MAX_MANAGEMENT_FEE_RATE)) {
        throw new errors_1.ValidationError(`LoanBrokerSet: ManagementFeeRate must be between 0 and ${MAX_MANAGEMENT_FEE_RATE} inclusive`);
    }
    if (tx.DebtMaximum != null && new bignumber_js_1.default(tx.DebtMaximum).isLessThan(0)) {
        throw new errors_1.ValidationError('LoanBrokerSet: DebtMaximum must be a non-negative value');
    }
    if (tx.CoverRateMinimum != null &&
        (tx.CoverRateMinimum < 0 || tx.CoverRateMinimum > MAX_COVER_RATE_MINIMUM)) {
        throw new errors_1.ValidationError(`LoanBrokerSet: CoverRateMinimum must be between 0 and ${MAX_COVER_RATE_MINIMUM} inclusive`);
    }
    if (tx.CoverRateLiquidation != null &&
        (tx.CoverRateLiquidation < 0 ||
            tx.CoverRateLiquidation > MAX_COVER_RATE_LIQUIDATION)) {
        throw new errors_1.ValidationError(`LoanBrokerSet: CoverRateLiquidation must be between 0 and ${MAX_COVER_RATE_LIQUIDATION} inclusive`);
    }
    const coverRateMinimumValue = (_a = tx.CoverRateMinimum) !== null && _a !== void 0 ? _a : 0;
    const coverRateLiquidationValue = (_b = tx.CoverRateLiquidation) !== null && _b !== void 0 ? _b : 0;
    if ((coverRateMinimumValue === 0 && coverRateLiquidationValue !== 0) ||
        (coverRateMinimumValue !== 0 && coverRateLiquidationValue === 0)) {
        throw new errors_1.ValidationError('LoanBrokerSet: CoverRateMinimum and CoverRateLiquidation must both be zero or both be non-zero');
    }
}
exports.validateLoanBrokerSet = validateLoanBrokerSet;
//# sourceMappingURL=loanBrokerSet.js.map