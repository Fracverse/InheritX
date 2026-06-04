"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.autofillBatchTxn = exports.handleDeliverMax = exports.checkAccountDeleteBlockers = exports.setLatestValidatedLedgerSequence = exports.getTransactionFee = exports.setNextValidSequenceNumber = exports.setValidAddresses = exports.txNeedsNetworkID = void 0;
const bignumber_js_1 = __importDefault(require("bignumber.js"));
const ripple_address_codec_1 = require("ripple-address-codec");
const errors_1 = require("../errors");
const utils_1 = require("../utils");
const getFeeXrp_1 = __importDefault(require("./getFeeXrp"));
const LEDGER_OFFSET = 20;
const RESTRICTED_NETWORKS = 1024;
const REQUIRED_NETWORKID_VERSION = '1.11.0';
function isNotLaterRippledVersion(source, target) {
    if (source === target) {
        return true;
    }
    const sourceDecomp = source.split('.');
    const targetDecomp = target.split('.');
    const sourceMajor = parseInt(sourceDecomp[0], 10);
    const sourceMinor = parseInt(sourceDecomp[1], 10);
    const targetMajor = parseInt(targetDecomp[0], 10);
    const targetMinor = parseInt(targetDecomp[1], 10);
    if (sourceMajor !== targetMajor) {
        return sourceMajor < targetMajor;
    }
    if (sourceMinor !== targetMinor) {
        return sourceMinor < targetMinor;
    }
    const sourcePatch = sourceDecomp[2].split('-');
    const targetPatch = targetDecomp[2].split('-');
    const sourcePatchVersion = parseInt(sourcePatch[0], 10);
    const targetPatchVersion = parseInt(targetPatch[0], 10);
    if (sourcePatchVersion !== targetPatchVersion) {
        return sourcePatchVersion < targetPatchVersion;
    }
    if (sourcePatch.length !== targetPatch.length) {
        return sourcePatch.length > targetPatch.length;
    }
    if (sourcePatch.length === 2) {
        if (!sourcePatch[1][0].startsWith(targetPatch[1][0])) {
            return sourcePatch[1] < targetPatch[1];
        }
        if (sourcePatch[1].startsWith('b')) {
            return (parseInt(sourcePatch[1].slice(1), 10) <
                parseInt(targetPatch[1].slice(1), 10));
        }
        return (parseInt(sourcePatch[1].slice(2), 10) <
            parseInt(targetPatch[1].slice(2), 10));
    }
    return false;
}
function txNeedsNetworkID(client) {
    if (client.networkID !== undefined &&
        client.networkID > RESTRICTED_NETWORKS) {
        if (client.buildVersion &&
            isNotLaterRippledVersion(REQUIRED_NETWORKID_VERSION, client.buildVersion)) {
            return true;
        }
    }
    return false;
}
exports.txNeedsNetworkID = txNeedsNetworkID;
function setValidAddresses(tx) {
    validateAccountAddress(tx, 'Account', 'SourceTag');
    if (tx['Destination'] != null) {
        validateAccountAddress(tx, 'Destination', 'DestinationTag');
    }
    convertToClassicAddress(tx, 'Authorize');
    convertToClassicAddress(tx, 'Unauthorize');
    convertToClassicAddress(tx, 'Owner');
    convertToClassicAddress(tx, 'RegularKey');
}
exports.setValidAddresses = setValidAddresses;
function validateAccountAddress(tx, accountField, tagField) {
    const { classicAccount, tag } = getClassicAccountAndTag(tx[accountField]);
    tx[accountField] = classicAccount;
    if (tag != null && tag !== false) {
        if (tx[tagField] && tx[tagField] !== tag) {
            throw new errors_1.ValidationError(`The ${tagField}, if present, must match the tag of the ${accountField} X-address`);
        }
        tx[tagField] = tag;
    }
}
function getClassicAccountAndTag(account, expectedTag) {
    if ((0, ripple_address_codec_1.isValidXAddress)(account)) {
        const classic = (0, ripple_address_codec_1.xAddressToClassicAddress)(account);
        if (expectedTag != null && classic.tag !== expectedTag) {
            throw new errors_1.ValidationError('address includes a tag that does not match the tag specified in the transaction');
        }
        return {
            classicAccount: classic.classicAddress,
            tag: classic.tag,
        };
    }
    return {
        classicAccount: account,
        tag: expectedTag,
    };
}
function convertToClassicAddress(tx, fieldName) {
    const account = tx[fieldName];
    if (typeof account === 'string') {
        const { classicAccount } = getClassicAccountAndTag(account);
        tx[fieldName] = classicAccount;
    }
}
function getNextValidSequenceNumber(client, account) {
    return __awaiter(this, void 0, void 0, function* () {
        const request = {
            command: 'account_info',
            account,
            ledger_index: 'current',
        };
        const data = yield client.request(request);
        return data.result.account_data.Sequence;
    });
}
function setNextValidSequenceNumber(client, tx) {
    return __awaiter(this, void 0, void 0, function* () {
        tx.Sequence = yield getNextValidSequenceNumber(client, tx.Account);
    });
}
exports.setNextValidSequenceNumber = setNextValidSequenceNumber;
function fetchOwnerReserveFee(client) {
    var _a;
    return __awaiter(this, void 0, void 0, function* () {
        const response = yield client.request({ command: 'server_state' });
        const fee = (_a = response.result.state.validated_ledger) === null || _a === void 0 ? void 0 : _a.reserve_inc;
        if (fee == null) {
            return Promise.reject(new Error('Could not fetch Owner Reserve.'));
        }
        return new bignumber_js_1.default(fee);
    });
}
function fetchCounterPartySignersCount(client, tx) {
    var _a, _b;
    return __awaiter(this, void 0, void 0, function* () {
        let counterParty = tx.Counterparty;
        if (counterParty == null) {
            if (tx.LoanBrokerID == null) {
                throw new errors_1.ValidationError('LoanBrokerID is required for LoanSet transaction');
            }
            const resp = (yield client.request({
                command: 'ledger_entry',
                index: tx.LoanBrokerID,
                ledger_index: 'validated',
            })).result.node;
            counterParty = resp.Owner;
        }
        const signerListRequest = {
            command: 'account_info',
            account: counterParty,
            ledger_index: 'validated',
            signer_lists: true,
        };
        const signerListResponse = yield client.request(signerListRequest);
        const signerList = (_a = signerListResponse.result.signer_lists) === null || _a === void 0 ? void 0 : _a[0];
        return (_b = signerList === null || signerList === void 0 ? void 0 : signerList.SignerEntries.length) !== null && _b !== void 0 ? _b : 1;
    });
}
function calculateFeePerTransactionType(client, tx, signersCount = 0) {
    return __awaiter(this, void 0, void 0, function* () {
        const netFeeXRP = yield (0, getFeeXrp_1.default)(client);
        const netFeeDrops = (0, utils_1.xrpToDrops)(netFeeXRP);
        let baseFee = new bignumber_js_1.default(netFeeDrops);
        const isSpecialTxCost = [
            'AccountDelete',
            'AMMCreate',
            'VaultCreate',
        ].includes(tx.TransactionType);
        if (tx.TransactionType === 'EscrowFinish' && tx.Fulfillment != null) {
            const fulfillmentBytesSize = Math.ceil(tx.Fulfillment.length / 2);
            baseFee = new bignumber_js_1.default(scaleValue(netFeeDrops, 33 + fulfillmentBytesSize / 16));
        }
        else if (isSpecialTxCost) {
            baseFee = yield fetchOwnerReserveFee(client);
        }
        else if (tx.TransactionType === 'Batch') {
            const rawTxFees = yield tx.RawTransactions.reduce((acc, rawTxn) => __awaiter(this, void 0, void 0, function* () {
                const resolvedAcc = yield acc;
                const fee = yield calculateFeePerTransactionType(client, rawTxn.RawTransaction);
                return bignumber_js_1.default.sum(resolvedAcc, fee);
            }), Promise.resolve(new bignumber_js_1.default(0)));
            baseFee = bignumber_js_1.default.sum(baseFee.times(2), rawTxFees);
        }
        if (signersCount > 0) {
            baseFee = bignumber_js_1.default.sum(baseFee, scaleValue(netFeeDrops, signersCount));
        }
        if (tx.TransactionType === 'LoanSet') {
            const counterPartySignersCount = yield fetchCounterPartySignersCount(client, tx);
            baseFee = bignumber_js_1.default.sum(baseFee, scaleValue(netFeeDrops, counterPartySignersCount));
            console.warn(`For LoanSet transaction the auto calculated Fee accounts for total number of signers the counterparty has to avoid transaction failure.`);
        }
        const maxFeeDrops = (0, utils_1.xrpToDrops)(client.maxFeeXRP);
        const totalFee = isSpecialTxCost
            ? baseFee
            : bignumber_js_1.default.min(baseFee, maxFeeDrops);
        return totalFee.dp(0, bignumber_js_1.default.ROUND_CEIL);
    });
}
function getTransactionFee(client, tx, signersCount = 0) {
    return __awaiter(this, void 0, void 0, function* () {
        const fee = yield calculateFeePerTransactionType(client, tx, signersCount);
        tx.Fee = fee.toString(10);
    });
}
exports.getTransactionFee = getTransactionFee;
function scaleValue(value, multiplier) {
    return new bignumber_js_1.default(value).times(multiplier).toString();
}
function setLatestValidatedLedgerSequence(client, tx) {
    return __awaiter(this, void 0, void 0, function* () {
        const ledgerSequence = yield client.getLedgerIndex();
        tx.LastLedgerSequence = ledgerSequence + LEDGER_OFFSET;
    });
}
exports.setLatestValidatedLedgerSequence = setLatestValidatedLedgerSequence;
function checkAccountDeleteBlockers(client, tx) {
    return __awaiter(this, void 0, void 0, function* () {
        const request = {
            command: 'account_objects',
            account: tx.Account,
            ledger_index: 'validated',
            deletion_blockers_only: true,
        };
        const response = yield client.request(request);
        return new Promise((resolve, reject) => {
            if (response.result.account_objects.length > 0) {
                reject(new errors_1.XrplError(`Account ${tx.Account} cannot be deleted; there are Escrows, PayChannels, RippleStates, or Checks associated with the account.`, response.result.account_objects));
            }
            resolve();
        });
    });
}
exports.checkAccountDeleteBlockers = checkAccountDeleteBlockers;
function handleDeliverMax(tx) {
    var _a;
    if (tx.DeliverMax != null) {
        (_a = tx.Amount) !== null && _a !== void 0 ? _a : (tx.Amount = tx.DeliverMax);
        if (tx.Amount != null && tx.Amount !== tx.DeliverMax) {
            throw new errors_1.ValidationError('PaymentTransaction: Amount and DeliverMax fields must be identical when both are provided');
        }
        delete tx.DeliverMax;
    }
}
exports.handleDeliverMax = handleDeliverMax;
function autofillBatchTxn(client, tx) {
    return __awaiter(this, void 0, void 0, function* () {
        const accountSequences = {};
        for (const rawTxn of tx.RawTransactions) {
            const txn = rawTxn.RawTransaction;
            if (txn.Sequence == null && txn.TicketSequence == null) {
                if (txn.Account in accountSequences) {
                    txn.Sequence = accountSequences[txn.Account];
                    accountSequences[txn.Account] += 1;
                }
                else {
                    const nextSequence = yield getNextValidSequenceNumber(client, txn.Account);
                    const sequence = txn.Account === tx.Account ? nextSequence + 1 : nextSequence;
                    accountSequences[txn.Account] = sequence + 1;
                    txn.Sequence = sequence;
                }
            }
            if (txn.Fee == null) {
                txn.Fee = '0';
            }
            else if (txn.Fee !== '0') {
                throw new errors_1.XrplError('Must have `Fee of "0" in inner Batch transaction.');
            }
            if (txn.SigningPubKey == null) {
                txn.SigningPubKey = '';
            }
            else if (txn.SigningPubKey !== '') {
                throw new errors_1.XrplError('Must have `SigningPubKey` of "" in inner Batch transaction.');
            }
            if (txn.TxnSignature != null) {
                throw new errors_1.XrplError('Must not have `TxnSignature` in inner Batch transaction.');
            }
            if (txn.Signers != null) {
                throw new errors_1.XrplError('Must not have `Signers` in inner Batch transaction.');
            }
            if (txn.NetworkID == null && txNeedsNetworkID(client)) {
                txn.NetworkID = client.networkID;
            }
        }
    });
}
exports.autofillBatchTxn = autofillBatchTxn;
//# sourceMappingURL=autofill.js.map