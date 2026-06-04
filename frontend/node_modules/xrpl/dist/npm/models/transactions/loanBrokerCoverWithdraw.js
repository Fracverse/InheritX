"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateLoanBrokerCoverWithdraw = void 0;
const errors_1 = require("../../errors");
const common_1 = require("./common");
function validateLoanBrokerCoverWithdraw(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateRequiredField)(tx, 'LoanBrokerID', common_1.isString);
    (0, common_1.validateRequiredField)(tx, 'Amount', common_1.isAmount);
    (0, common_1.validateOptionalField)(tx, 'Destination', common_1.isAccount);
    (0, common_1.validateOptionalField)(tx, 'DestinationTag', common_1.isNumber);
    if (!(0, common_1.isLedgerEntryId)(tx.LoanBrokerID)) {
        throw new errors_1.ValidationError(`LoanBrokerCoverWithdraw: LoanBrokerID must be 64 characters hexadecimal string`);
    }
}
exports.validateLoanBrokerCoverWithdraw = validateLoanBrokerCoverWithdraw;
//# sourceMappingURL=loanBrokerCoverWithdraw.js.map