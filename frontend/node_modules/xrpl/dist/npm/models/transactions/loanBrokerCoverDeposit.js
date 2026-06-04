"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateLoanBrokerCoverDeposit = void 0;
const errors_1 = require("../../errors");
const common_1 = require("./common");
function validateLoanBrokerCoverDeposit(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateRequiredField)(tx, 'LoanBrokerID', common_1.isString);
    (0, common_1.validateRequiredField)(tx, 'Amount', common_1.isAmount);
    if (!(0, common_1.isLedgerEntryId)(tx.LoanBrokerID)) {
        throw new errors_1.ValidationError(`LoanBrokerCoverDeposit: LoanBrokerID must be 64 characters hexadecimal string`);
    }
}
exports.validateLoanBrokerCoverDeposit = validateLoanBrokerCoverDeposit;
//# sourceMappingURL=loanBrokerCoverDeposit.js.map