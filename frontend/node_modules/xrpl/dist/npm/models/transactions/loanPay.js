"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateLoanPay = exports.LoanPayFlags = void 0;
const errors_1 = require("../../errors");
const utils_1 = require("../utils");
const common_1 = require("./common");
var LoanPayFlags;
(function (LoanPayFlags) {
    LoanPayFlags[LoanPayFlags["tfLoanOverpayment"] = 65536] = "tfLoanOverpayment";
    LoanPayFlags[LoanPayFlags["tfLoanFullPayment"] = 131072] = "tfLoanFullPayment";
    LoanPayFlags[LoanPayFlags["tfLoanLatePayment"] = 262144] = "tfLoanLatePayment";
})(LoanPayFlags || (exports.LoanPayFlags = LoanPayFlags = {}));
function validateLoanPay(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateRequiredField)(tx, 'LoanID', common_1.isString);
    (0, common_1.validateRequiredField)(tx, 'Amount', common_1.isAmount);
    if (!(0, common_1.isLedgerEntryId)(tx.LoanID)) {
        throw new errors_1.ValidationError(`LoanPay: LoanID must be 64 characters hexadecimal string`);
    }
    if (typeof tx.Flags === 'number') {
        const flagsSet = [
            (0, utils_1.isFlagEnabled)(tx.Flags, LoanPayFlags.tfLoanLatePayment),
            (0, utils_1.isFlagEnabled)(tx.Flags, LoanPayFlags.tfLoanFullPayment),
            (0, utils_1.isFlagEnabled)(tx.Flags, LoanPayFlags.tfLoanOverpayment),
        ].filter(Boolean).length;
        if (flagsSet > 1) {
            throw new errors_1.ValidationError('LoanPay: Only one of tfLoanLatePayment, tfLoanFullPayment, or tfLoanOverpayment flags can be set');
        }
    }
    else if (tx.Flags != null && typeof tx.Flags === 'object') {
        const flags = tx.Flags;
        const flagsSet = [
            flags.tfLoanLatePayment,
            flags.tfLoanFullPayment,
            flags.tfLoanOverpayment,
        ].filter(Boolean).length;
        if (flagsSet > 1) {
            throw new errors_1.ValidationError('LoanPay: Only one of tfLoanLatePayment, tfLoanFullPayment, or tfLoanOverpayment flags can be set');
        }
    }
}
exports.validateLoanPay = validateLoanPay;
//# sourceMappingURL=loanPay.js.map