"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateLoanManage = exports.LoanManageFlags = void 0;
const errors_1 = require("../../errors");
const common_1 = require("./common");
var LoanManageFlags;
(function (LoanManageFlags) {
    LoanManageFlags[LoanManageFlags["tfLoanDefault"] = 65536] = "tfLoanDefault";
    LoanManageFlags[LoanManageFlags["tfLoanImpair"] = 131072] = "tfLoanImpair";
    LoanManageFlags[LoanManageFlags["tfLoanUnimpair"] = 262144] = "tfLoanUnimpair";
})(LoanManageFlags || (exports.LoanManageFlags = LoanManageFlags = {}));
function validateLoanManage(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateRequiredField)(tx, 'LoanID', common_1.isString);
    if (!(0, common_1.isLedgerEntryId)(tx.LoanID)) {
        throw new errors_1.ValidationError(`LoanManage: LoanID must be 64 characters hexadecimal string`);
    }
    const txFlags = tx.Flags;
    if (txFlags == null) {
        return;
    }
    let flags = 0;
    if (typeof txFlags === 'number') {
        flags = txFlags;
    }
    else {
        if (txFlags.tfLoanImpair) {
            flags |= LoanManageFlags.tfLoanImpair;
        }
        if (txFlags.tfLoanUnimpair) {
            flags |= LoanManageFlags.tfLoanUnimpair;
        }
    }
    if ((flags & LoanManageFlags.tfLoanImpair) === LoanManageFlags.tfLoanImpair &&
        (flags & LoanManageFlags.tfLoanUnimpair) === LoanManageFlags.tfLoanUnimpair) {
        throw new errors_1.ValidationError('LoanManage: tfLoanImpair and tfLoanUnimpair cannot both be present');
    }
}
exports.validateLoanManage = validateLoanManage;
//# sourceMappingURL=loanManage.js.map