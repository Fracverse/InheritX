"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isDomainID = exports.containsDuplicates = exports.validateCredentialsList = exports.validateCredentialType = exports.parseAmountValue = exports.validateBaseTransaction = exports.GlobalFlags = exports.validateOptionalField = exports.validateRequiredField = exports.validateHexMetadata = exports.isLedgerEntryId = exports.isArray = exports.isXChainBridge = exports.isTokenAmount = exports.isAmount = exports.isAccount = exports.isClawbackAmount = exports.isMPTAmount = exports.isAuthorizeCredential = exports.isIssuedCurrencyAmount = exports.isIssuedCurrency = exports.isCurrency = exports.isXRPLNumber = exports.isValue = exports.isNull = exports.isNumber = exports.isString = exports.isRecord = exports.VAULT_DATA_MAX_BYTE_LENGTH = exports.MAX_AUTHORIZED_CREDENTIALS = void 0;
const utils_1 = require("@xrplf/isomorphic/utils");
const ripple_address_codec_1 = require("ripple-address-codec");
const ripple_binary_codec_1 = require("ripple-binary-codec");
const errors_1 = require("../../errors");
const utils_2 = require("../utils");
const MEMO_SIZE = 3;
exports.MAX_AUTHORIZED_CREDENTIALS = 8;
const MAX_CREDENTIAL_BYTE_LENGTH = 64;
const MAX_CREDENTIAL_TYPE_LENGTH = MAX_CREDENTIAL_BYTE_LENGTH * 2;
const SHA_512_HALF_LENGTH = 64;
exports.VAULT_DATA_MAX_BYTE_LENGTH = 256;
function isMemo(obj) {
    if (!isRecord(obj)) {
        return false;
    }
    const memo = obj.Memo;
    if (!isRecord(memo)) {
        return false;
    }
    const size = Object.keys(memo).length;
    const validData = memo.MemoData == null || (isString(memo.MemoData) && (0, utils_2.isHex)(memo.MemoData));
    const validFormat = memo.MemoFormat == null ||
        (isString(memo.MemoFormat) && (0, utils_2.isHex)(memo.MemoFormat));
    const validType = memo.MemoType == null || (isString(memo.MemoType) && (0, utils_2.isHex)(memo.MemoType));
    return (size >= 1 &&
        size <= MEMO_SIZE &&
        validData &&
        validFormat &&
        validType &&
        (0, utils_2.onlyHasFields)(memo, ['MemoFormat', 'MemoData', 'MemoType']));
}
const SIGNER_SIZE = 3;
function isSigner(obj) {
    if (!isRecord(obj)) {
        return false;
    }
    const signer = obj.Signer;
    if (!isRecord(signer)) {
        return false;
    }
    return (Object.keys(signer).length === SIGNER_SIZE &&
        isString(signer.Account) &&
        isString(signer.TxnSignature) &&
        isString(signer.SigningPubKey));
}
const XRP_CURRENCY_SIZE = 1;
const MPT_CURRENCY_SIZE = 1;
const ISSUE_CURRENCY_SIZE = 2;
const MPT_CURRENCY_AMOUNT_SIZE = 2;
const ISSUED_CURRENCY_AMOUNT_SIZE = 3;
const XCHAIN_BRIDGE_SIZE = 4;
const AUTHORIZE_CREDENTIAL_SIZE = 1;
function isRecord(value) {
    return value !== null && typeof value === 'object' && !Array.isArray(value);
}
exports.isRecord = isRecord;
function isString(str) {
    return typeof str === 'string';
}
exports.isString = isString;
function isNumber(num) {
    return typeof num === 'number';
}
exports.isNumber = isNumber;
function isNull(inp) {
    return inp == null;
}
exports.isNull = isNull;
function isValue(value) {
    const isValueInternal = (inp) => inp === value;
    return isValueInternal;
}
exports.isValue = isValue;
function isXRPLNumber(value) {
    return (typeof value === 'string' &&
        /^[-+]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][-+]?\d+)?$/u.test(value.trim()));
}
exports.isXRPLNumber = isXRPLNumber;
function isCurrency(input) {
    return (isRecord(input) &&
        ((Object.keys(input).length === ISSUE_CURRENCY_SIZE &&
            isString(input.issuer) &&
            isString(input.currency)) ||
            (Object.keys(input).length === XRP_CURRENCY_SIZE &&
                input.currency === 'XRP') ||
            (Object.keys(input).length === MPT_CURRENCY_SIZE &&
                isString(input.mpt_issuance_id))));
}
exports.isCurrency = isCurrency;
function isIssuedCurrency(input) {
    return (isRecord(input) &&
        ((Object.keys(input).length === ISSUE_CURRENCY_SIZE &&
            isString(input.issuer) &&
            isString(input.currency)) ||
            (Object.keys(input).length === XRP_CURRENCY_SIZE &&
                input.currency === 'XRP')));
}
exports.isIssuedCurrency = isIssuedCurrency;
function isIssuedCurrencyAmount(input) {
    return (isRecord(input) &&
        Object.keys(input).length === ISSUED_CURRENCY_AMOUNT_SIZE &&
        isString(input.value) &&
        isString(input.issuer) &&
        isString(input.currency));
}
exports.isIssuedCurrencyAmount = isIssuedCurrencyAmount;
function isAuthorizeCredential(input) {
    return (isRecord(input) &&
        isRecord(input.Credential) &&
        Object.keys(input).length === AUTHORIZE_CREDENTIAL_SIZE &&
        typeof input.Credential.CredentialType === 'string' &&
        typeof input.Credential.Issuer === 'string');
}
exports.isAuthorizeCredential = isAuthorizeCredential;
function isMPTAmount(input) {
    return (isRecord(input) &&
        Object.keys(input).length === MPT_CURRENCY_AMOUNT_SIZE &&
        typeof input.value === 'string' &&
        typeof input.mpt_issuance_id === 'string');
}
exports.isMPTAmount = isMPTAmount;
function isClawbackAmount(input) {
    return isIssuedCurrencyAmount(input) || isMPTAmount(input);
}
exports.isClawbackAmount = isClawbackAmount;
function isAccount(account) {
    return (typeof account === 'string' &&
        ((0, ripple_address_codec_1.isValidClassicAddress)(account) || (0, ripple_address_codec_1.isValidXAddress)(account)));
}
exports.isAccount = isAccount;
function isAmount(amount) {
    return (typeof amount === 'string' ||
        isIssuedCurrencyAmount(amount) ||
        isMPTAmount(amount));
}
exports.isAmount = isAmount;
function isTokenAmount(amount) {
    return isIssuedCurrencyAmount(amount) || isMPTAmount(amount);
}
exports.isTokenAmount = isTokenAmount;
function isXChainBridge(input) {
    return (isRecord(input) &&
        Object.keys(input).length === XCHAIN_BRIDGE_SIZE &&
        typeof input.LockingChainDoor === 'string' &&
        isIssuedCurrency(input.LockingChainIssue) &&
        typeof input.IssuingChainDoor === 'string' &&
        isIssuedCurrency(input.IssuingChainIssue));
}
exports.isXChainBridge = isXChainBridge;
function isArray(input) {
    return input != null && Array.isArray(input);
}
exports.isArray = isArray;
function isLedgerEntryId(input) {
    return isString(input) && (0, utils_2.isHex)(input) && input.length === SHA_512_HALF_LENGTH;
}
exports.isLedgerEntryId = isLedgerEntryId;
function validateHexMetadata(input, lengthUpto) {
    return (isString(input) &&
        (0, utils_2.isHex)(input) &&
        input.length > 0 &&
        input.length <= lengthUpto);
}
exports.validateHexMetadata = validateHexMetadata;
function validateRequiredField(tx, param, checkValidity, errorOpts = {}) {
    var _a, _b;
    const paramNameStr = (_a = errorOpts.paramName) !== null && _a !== void 0 ? _a : param;
    const txType = (_b = errorOpts.txType) !== null && _b !== void 0 ? _b : tx.TransactionType;
    if (tx[param] == null) {
        throw new errors_1.ValidationError(`${txType}: missing field ${String(paramNameStr)}`);
    }
    if (!checkValidity(tx[param])) {
        throw new errors_1.ValidationError(`${txType}: invalid field ${String(paramNameStr)}`);
    }
}
exports.validateRequiredField = validateRequiredField;
function validateOptionalField(tx, param, checkValidity, errorOpts = {}) {
    var _a, _b;
    const paramNameStr = (_a = errorOpts.paramName) !== null && _a !== void 0 ? _a : param;
    const txType = (_b = errorOpts.txType) !== null && _b !== void 0 ? _b : tx.TransactionType;
    if (tx[param] !== undefined && !checkValidity(tx[param])) {
        throw new errors_1.ValidationError(`${txType}: invalid field ${String(paramNameStr)}`);
    }
}
exports.validateOptionalField = validateOptionalField;
var GlobalFlags;
(function (GlobalFlags) {
    GlobalFlags[GlobalFlags["tfInnerBatchTxn"] = 1073741824] = "tfInnerBatchTxn";
})(GlobalFlags || (exports.GlobalFlags = GlobalFlags = {}));
function validateBaseTransaction(common) {
    if (!isRecord(common)) {
        throw new errors_1.ValidationError('BaseTransaction: invalid, expected a valid object');
    }
    if (common.TransactionType === undefined) {
        throw new errors_1.ValidationError('BaseTransaction: missing field TransactionType');
    }
    if (typeof common.TransactionType !== 'string') {
        throw new errors_1.ValidationError('BaseTransaction: TransactionType not string');
    }
    if (!ripple_binary_codec_1.TRANSACTION_TYPES.includes(common.TransactionType)) {
        throw new errors_1.ValidationError(`BaseTransaction: Unknown TransactionType ${common.TransactionType}`);
    }
    validateRequiredField(common, 'Account', isString);
    validateOptionalField(common, 'Fee', isString);
    validateOptionalField(common, 'Sequence', isNumber);
    validateOptionalField(common, 'AccountTxnID', isString);
    validateOptionalField(common, 'LastLedgerSequence', isNumber);
    const memos = common.Memos;
    if (memos != null && (!isArray(memos) || !memos.every(isMemo))) {
        throw new errors_1.ValidationError('BaseTransaction: invalid Memos');
    }
    const signers = common.Signers;
    if (signers != null &&
        (!isArray(signers) || signers.length === 0 || !signers.every(isSigner))) {
        throw new errors_1.ValidationError('BaseTransaction: invalid Signers');
    }
    validateOptionalField(common, 'SourceTag', isNumber);
    validateOptionalField(common, 'SigningPubKey', isString);
    validateOptionalField(common, 'TicketSequence', isNumber);
    validateOptionalField(common, 'TxnSignature', isString);
    validateOptionalField(common, 'NetworkID', isNumber);
    validateOptionalField(common, 'Delegate', isAccount);
    const delegate = common.Delegate;
    if (delegate != null && delegate === common.Account) {
        throw new errors_1.ValidationError('BaseTransaction: Account and Delegate addresses cannot be the same');
    }
}
exports.validateBaseTransaction = validateBaseTransaction;
function parseAmountValue(amount) {
    if (!isAmount(amount)) {
        return NaN;
    }
    if (typeof amount === 'string') {
        return parseFloat(amount);
    }
    return parseFloat(amount.value);
}
exports.parseAmountValue = parseAmountValue;
function validateCredentialType(tx) {
    if (typeof tx.TransactionType !== 'string') {
        throw new errors_1.ValidationError('Invalid TransactionType');
    }
    if (tx.CredentialType === undefined) {
        throw new errors_1.ValidationError(`${tx.TransactionType}: missing field CredentialType`);
    }
    if (!isString(tx.CredentialType)) {
        throw new errors_1.ValidationError(`${tx.TransactionType}: CredentialType must be a string`);
    }
    if (tx.CredentialType.length === 0) {
        throw new errors_1.ValidationError(`${tx.TransactionType}: CredentialType cannot be an empty string`);
    }
    else if (tx.CredentialType.length > MAX_CREDENTIAL_TYPE_LENGTH) {
        throw new errors_1.ValidationError(`${tx.TransactionType}: CredentialType length cannot be > ${MAX_CREDENTIAL_TYPE_LENGTH}`);
    }
    if (!utils_1.HEX_REGEX.test(tx.CredentialType)) {
        throw new errors_1.ValidationError(`${tx.TransactionType}: CredentialType must be encoded in hex`);
    }
}
exports.validateCredentialType = validateCredentialType;
function validateCredentialsList(credentials, transactionType, isStringID, maxCredentials) {
    if (credentials == null) {
        return;
    }
    if (!isArray(credentials)) {
        throw new errors_1.ValidationError(`${transactionType}: Credentials must be an array`);
    }
    if (credentials.length > maxCredentials) {
        throw new errors_1.ValidationError(`${transactionType}: Credentials length cannot exceed ${maxCredentials} elements`);
    }
    else if (credentials.length === 0) {
        throw new errors_1.ValidationError(`${transactionType}: Credentials cannot be an empty array`);
    }
    credentials.forEach((credential) => {
        if (isStringID) {
            if (!isString(credential)) {
                throw new errors_1.ValidationError(`${transactionType}: Invalid Credentials ID list format`);
            }
        }
        else if (!isAuthorizeCredential(credential)) {
            throw new errors_1.ValidationError(`${transactionType}: Invalid Credentials format`);
        }
    });
    if (containsDuplicates(credentials)) {
        throw new errors_1.ValidationError(`${transactionType}: Credentials cannot contain duplicate elements`);
    }
}
exports.validateCredentialsList = validateCredentialsList;
function isAuthorizeCredentialArray(list) {
    return typeof list[0] !== 'string';
}
function containsDuplicates(objectList) {
    if (typeof objectList[0] === 'string') {
        const objSet = new Set(objectList.map((obj) => JSON.stringify(obj)));
        return objSet.size !== objectList.length;
    }
    const seen = new Set();
    if (isAuthorizeCredentialArray(objectList)) {
        for (const item of objectList) {
            const key = `${item.Credential.Issuer}-${item.Credential.CredentialType}`;
            if (seen.has(key)) {
                return true;
            }
            seen.add(key);
        }
    }
    return false;
}
exports.containsDuplicates = containsDuplicates;
const _DOMAIN_ID_LENGTH = 64;
function isDomainID(domainID) {
    return (isString(domainID) &&
        domainID.length === _DOMAIN_ID_LENGTH &&
        (0, utils_2.isHex)(domainID));
}
exports.isDomainID = isDomainID;
//# sourceMappingURL=common.js.map