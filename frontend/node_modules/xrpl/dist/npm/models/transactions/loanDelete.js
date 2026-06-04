"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateLoanDelete = void 0;
const errors_1 = require("../../errors");
const common_1 = require("./common");
function validateLoanDelete(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateRequiredField)(tx, 'LoanID', common_1.isString);
    if (!(0, common_1.isLedgerEntryId)(tx.LoanID)) {
        throw new errors_1.ValidationError(`LoanDelete: LoanID must be 64 characters hexadecimal string`);
    }
}
exports.validateLoanDelete = validateLoanDelete;
//# sourceMappingURL=loanDelete.js.map