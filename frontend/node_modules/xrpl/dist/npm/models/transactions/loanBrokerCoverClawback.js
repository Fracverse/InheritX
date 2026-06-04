"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateLoanBrokerCoverClawback = void 0;
const bignumber_js_1 = __importDefault(require("bignumber.js"));
const errors_1 = require("../../errors");
const common_1 = require("./common");
function validateLoanBrokerCoverClawback(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateOptionalField)(tx, 'LoanBrokerID', common_1.isString);
    (0, common_1.validateOptionalField)(tx, 'Amount', common_1.isTokenAmount);
    if (tx.LoanBrokerID != null && !(0, common_1.isLedgerEntryId)(tx.LoanBrokerID)) {
        throw new errors_1.ValidationError(`LoanBrokerCoverClawback: LoanBrokerID must be 64 characters hexadecimal string`);
    }
    if (tx.Amount != null && new bignumber_js_1.default(tx.Amount.value).isLessThan(0)) {
        throw new errors_1.ValidationError(`LoanBrokerCoverClawback: Amount must be >= 0`);
    }
    if (tx.LoanBrokerID == null && tx.Amount == null) {
        throw new errors_1.ValidationError(`LoanBrokerCoverClawback: Either LoanBrokerID or Amount is required`);
    }
}
exports.validateLoanBrokerCoverClawback = validateLoanBrokerCoverClawback;
//# sourceMappingURL=loanBrokerCoverClawback.js.map