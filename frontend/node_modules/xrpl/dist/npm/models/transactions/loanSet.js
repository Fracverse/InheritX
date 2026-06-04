"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateLoanSet = exports.LoanSetFlags = void 0;
const errors_1 = require("../../errors");
const common_1 = require("./common");
const MAX_DATA_LENGTH = 512;
const MAX_OVER_PAYMENT_FEE_RATE = 100000;
const MAX_INTEREST_RATE = 100000;
const MAX_LATE_INTEREST_RATE = 100000;
const MAX_CLOSE_INTEREST_RATE = 100000;
const MAX_OVER_PAYMENT_INTEREST_RATE = 100000;
const MIN_PAYMENT_INTERVAL = 60;
var LoanSetFlags;
(function (LoanSetFlags) {
    LoanSetFlags[LoanSetFlags["tfLoanOverpayment"] = 65536] = "tfLoanOverpayment";
})(LoanSetFlags || (exports.LoanSetFlags = LoanSetFlags = {}));
function validateLoanSet(tx) {
    (0, common_1.validateBaseTransaction)(tx);
    (0, common_1.validateRequiredField)(tx, 'LoanBrokerID', common_1.isString);
    (0, common_1.validateRequiredField)(tx, 'PrincipalRequested', common_1.isXRPLNumber);
    (0, common_1.validateOptionalField)(tx, 'CounterpartySignature', common_1.isRecord);
    (0, common_1.validateOptionalField)(tx, 'Data', common_1.isString);
    (0, common_1.validateOptionalField)(tx, 'Counterparty', common_1.isAccount);
    (0, common_1.validateOptionalField)(tx, 'LoanOriginationFee', common_1.isXRPLNumber);
    (0, common_1.validateOptionalField)(tx, 'LoanServiceFee', common_1.isXRPLNumber);
    (0, common_1.validateOptionalField)(tx, 'LatePaymentFee', common_1.isXRPLNumber);
    (0, common_1.validateOptionalField)(tx, 'ClosePaymentFee', common_1.isXRPLNumber);
    (0, common_1.validateOptionalField)(tx, 'OverpaymentFee', common_1.isNumber);
    (0, common_1.validateOptionalField)(tx, 'InterestRate', common_1.isNumber);
    (0, common_1.validateOptionalField)(tx, 'LateInterestRate', common_1.isNumber);
    (0, common_1.validateOptionalField)(tx, 'CloseInterestRate', common_1.isNumber);
    (0, common_1.validateOptionalField)(tx, 'OverpaymentInterestRate', common_1.isNumber);
    (0, common_1.validateOptionalField)(tx, 'PaymentTotal', common_1.isNumber);
    (0, common_1.validateOptionalField)(tx, 'PaymentInterval', common_1.isNumber);
    (0, common_1.validateOptionalField)(tx, 'GracePeriod', common_1.isNumber);
    if (!(0, common_1.isLedgerEntryId)(tx.LoanBrokerID)) {
        throw new errors_1.ValidationError(`LoanSet: LoanBrokerID must be 64 characters hexadecimal string`);
    }
    if (tx.Data != null && !(0, common_1.validateHexMetadata)(tx.Data, MAX_DATA_LENGTH)) {
        throw new errors_1.ValidationError(`LoanSet: Data must be a valid non-empty hex string up to ${MAX_DATA_LENGTH} characters`);
    }
    if (tx.OverpaymentFee != null &&
        (tx.OverpaymentFee < 0 || tx.OverpaymentFee > MAX_OVER_PAYMENT_FEE_RATE)) {
        throw new errors_1.ValidationError(`LoanSet: OverpaymentFee must be between 0 and ${MAX_OVER_PAYMENT_FEE_RATE} inclusive`);
    }
    if (tx.InterestRate != null &&
        (tx.InterestRate < 0 || tx.InterestRate > MAX_INTEREST_RATE)) {
        throw new errors_1.ValidationError(`LoanSet: InterestRate must be between 0 and ${MAX_INTEREST_RATE} inclusive`);
    }
    if (tx.LateInterestRate != null &&
        (tx.LateInterestRate < 0 || tx.LateInterestRate > MAX_LATE_INTEREST_RATE)) {
        throw new errors_1.ValidationError(`LoanSet: LateInterestRate must be between 0 and ${MAX_LATE_INTEREST_RATE} inclusive`);
    }
    if (tx.CloseInterestRate != null &&
        (tx.CloseInterestRate < 0 || tx.CloseInterestRate > MAX_CLOSE_INTEREST_RATE)) {
        throw new errors_1.ValidationError(`LoanSet: CloseInterestRate must be between 0 and ${MAX_CLOSE_INTEREST_RATE} inclusive`);
    }
    if (tx.OverpaymentInterestRate != null &&
        (tx.OverpaymentInterestRate < 0 ||
            tx.OverpaymentInterestRate > MAX_OVER_PAYMENT_INTEREST_RATE)) {
        throw new errors_1.ValidationError(`LoanSet: OverpaymentInterestRate must be between 0 and ${MAX_OVER_PAYMENT_INTEREST_RATE} inclusive`);
    }
    if (tx.PaymentInterval != null && tx.PaymentInterval < MIN_PAYMENT_INTERVAL) {
        throw new errors_1.ValidationError(`LoanSet: PaymentInterval must be greater than or equal to ${MIN_PAYMENT_INTERVAL}`);
    }
    if (tx.PaymentInterval != null &&
        tx.GracePeriod != null &&
        tx.GracePeriod > tx.PaymentInterval) {
        throw new errors_1.ValidationError(`LoanSet: GracePeriod must not be greater than PaymentInterval`);
    }
}
exports.validateLoanSet = validateLoanSet;
//# sourceMappingURL=loanSet.js.map