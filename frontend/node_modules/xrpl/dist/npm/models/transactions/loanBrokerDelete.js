"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateLoanBrokerDelete = void 0;
const errors_1 = require("../../errors");
const common_1 = require("./common");
function validateLoanBrokerDelete(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateRequiredField)(tx, 'LoanBrokerID', common_1.isString);
    if (!(0, common_1.isLedgerEntryId)(tx.LoanBrokerID)) {
        throw new errors_1.ValidationError(`LoanBrokerDelete: LoanBrokerID must be 64 characters hexadecimal string`);
    }
}
exports.validateLoanBrokerDelete = validateLoanBrokerDelete;
//# sourceMappingURL=loanBrokerDelete.js.map