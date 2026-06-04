"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.BLOCK_ACCESS_LIST_ITEM_COST = void 0;
exports.countBlockAccessListItems = countBlockAccessListItems;
exports.validateBlockAccessListGasLimit = validateBlockAccessListGasLimit;
exports.validateBlockAccessList = validateBlockAccessList;
exports.validateBlockAccessListJSONStructure = validateBlockAccessListJSONStructure;
exports.isAccountOrderOnlyViolation = isAccountOrderOnlyViolation;
exports.hashBlockAccessListFromJSON = hashBlockAccessListFromJSON;
exports.validateBlockAccessListHashFromJSON = validateBlockAccessListHashFromJSON;
exports.validateBlockAccessListStructure = validateBlockAccessListStructure;
exports.validateBlockAccessListHash = validateBlockAccessListHash;
exports.equalsBlockAccessList = equalsBlockAccessList;
const rlp_1 = require("@ethereumjs/rlp");
const sha3_js_1 = require("@noble/hashes/sha3.js");
const bytes_ts_1 = require("../bytes.js");
const constants_ts_1 = require("../constants.js");
const errors_ts_1 = require("../errors.js");
const index_ts_1 = require("./index.js");
/**
 * EIP-7928 gas cost attributed to each BAL item (one address or one storage key).
 *
 * @remarks Experimental (Amsterdam): may change on patch releases.
 */
exports.BLOCK_ACCESS_LIST_ITEM_COST = 2000;
/**
 * Counts BAL items per EIP-7928: `addresses + storage_keys`.
 * Uses the canonical {@link BlockLevelAccessList.raw()} view.
 *
 * @remarks Experimental (Amsterdam): may change on patch releases.
 */
function countBlockAccessListItems(bal) {
    let items = 0;
    for (const [, storageChanges, storageReads] of bal.raw()) {
        items += 1 + storageChanges.length + storageReads.length;
    }
    return items;
}
/**
 * Ensures `bal_items <= block_gas_limit // ITEM_COST` (EIP-7928).
 *
 * @throws if the access list exceeds the block gas-derived item cap
 * @remarks Experimental (Amsterdam): may change on patch releases.
 */
function validateBlockAccessListGasLimit(bal, blockGasLimit) {
    const maxItems = blockGasLimit / BigInt(exports.BLOCK_ACCESS_LIST_ITEM_COST);
    const items = BigInt(countBlockAccessListItems(bal));
    if (items > maxItems) {
        throwBlockAccessListGasLimitExceeded();
    }
}
/**
 * Validates canonical BAL structure and, when provided, the header hash commitment.
 *
 * @remarks Experimental (Amsterdam): may change on patch releases.
 */
function validateBlockAccessList(bal, blockAccessListHash) {
    validateBlockAccessListStructure(bal);
    if (blockAccessListHash !== undefined) {
        validateBlockAccessListHash(bal, blockAccessListHash);
    }
}
/**
 * Validates lexicographic ordering, uniqueness, and field constraints of a
 * block access list in JSON fixture / Engine API form.
 *
 * Use before {@link createBlockLevelAccessListFromJSON} so out-of-order or
 * duplicate accounts are not silently merged.
 *
 * @remarks Experimental (Amsterdam): may change on patch releases.
 */
function validateBlockAccessListJSONStructure(json) {
    const seenAddresses = new Set();
    for (const account of json) {
        validateBlockAccessListJSONAccount(account);
        if (seenAddresses.has(account.address)) {
            throwIncorrectBlockFormat('duplicate account in block access list');
        }
        seenAddresses.add(account.address);
    }
}
/**
 * True when accounts are out of lexicographic order in a way that indicates a full reorder
 * (e.g. `reverse_accounts()`), as opposed to a single appended account breaking sort at the end.
 *
 * @remarks Experimental (Amsterdam): may change on patch releases.
 */
function isAccountOrderOnlyViolation(json) {
    return countAccountOrderInversions(json) >= 2;
}
/**
 * `keccak256(rlp(bal))` using the JSON account order (not re-sorted), matching Engine API bytes.
 *
 * @remarks Experimental (Amsterdam): may change on patch releases.
 */
function hashBlockAccessListFromJSON(json) {
    const accounts = [];
    for (const account of json) {
        const entry = encodeAccountRawFromJSON(account);
        if (entry !== undefined) {
            accounts.push(entry);
        }
    }
    return (0, sha3_js_1.keccak_256)(rlp_1.RLP.encode(accounts));
}
/**
 * Verifies the header hash against the JSON-ordered RLP encoding.
 *
 * @remarks Experimental (Amsterdam): may change on patch releases.
 */
function validateBlockAccessListHashFromJSON(json, expectedHash) {
    validateBlockAccessListHashBytes(hashBlockAccessListFromJSON(json), expectedHash);
}
/**
 * Validates the canonical RLP-oriented structure of a {@link BlockLevelAccessList}.
 *
 * @remarks Experimental (Amsterdam): may change on patch releases.
 */
function validateBlockAccessListStructure(bal) {
    validateBlockAccessListRaw(bal.raw());
}
/**
 * Verifies `keccak256(rlp(bal))` matches the committed header hash.
 *
 * @remarks Experimental (Amsterdam): may change on patch releases.
 */
function validateBlockAccessListHash(bal, expectedHash) {
    validateBlockAccessListHashBytes(bal.hash(), expectedHash);
}
/**
 * Returns true when two access lists produce identical canonical RLP.
 *
 * @remarks Experimental (Amsterdam): may change on patch releases.
 */
function equalsBlockAccessList(a, b) {
    return (0, bytes_ts_1.equalsBytes)(a.serialize(), b.serialize());
}
function countAccountOrderInversions(json) {
    let inversions = 0;
    for (let i = 0; i < json.length - 1; i++) {
        if (compareLexicographicHex(json[i].address, json[i + 1].address) > 0) {
            inversions++;
        }
    }
    return inversions;
}
function validateBlockAccessListHashBytes(computed, expectedHash) {
    if (expectedHash.length !== 32) {
        throwInvalidBalHash(`expected 32-byte hash, received ${expectedHash.length} bytes`);
    }
    if (!(0, bytes_ts_1.equalsBytes)(computed, expectedHash)) {
        throwInvalidBalHash(`received=${(0, bytes_ts_1.bytesToHex)(expectedHash)} expected=${(0, bytes_ts_1.bytesToHex)(computed)}`);
    }
}
function encodeAccountRawFromJSON(account) {
    const bal = (0, index_ts_1.createBlockLevelAccessListFromJSON)([account]);
    return bal.raw()[0];
}
function validateBlockAccessListRaw(raw) {
    let prevAddress;
    for (const account of raw) {
        const [address, storageChanges, storageReads, balanceChanges, nonceChanges, codeChanges] = account;
        if (prevAddress === address) {
            throwIncorrectBlockFormat('duplicate account in block access list');
        }
        if (prevAddress !== undefined && compareLexicographicHex(prevAddress, address) > 0) {
            throwIncorrectBlockFormat('block access list accounts are not sorted');
        }
        prevAddress = address;
        if (address === constants_ts_1.SYSTEM_ADDRESS && !rawAccountHasChanges(account)) {
            throwInvalidBlockAccessList('system address must not be included without state changes');
        }
        validateStorageChanges(storageChanges);
        validateStorageReads(storageReads, storageChanges);
        validateSortedUniqueIndices(balanceChanges.map(([index]) => index), 'balance');
        validateSortedUniqueIndices(nonceChanges.map(([index]) => index), 'nonce');
        validateSortedUniqueIndices(codeChanges.map(([index]) => index), 'code');
    }
}
function validateBlockAccessListJSONAccount(account) {
    validateAddress(account.address);
    if (account.address === constants_ts_1.SYSTEM_ADDRESS && !accountHasChanges(account)) {
        throwInvalidBlockAccessList('system address must not be included without state changes');
    }
    let prevSlot;
    for (const slotChange of account.storageChanges) {
        if (prevSlot !== undefined && compareLexicographicHex(prevSlot, slotChange.slot) >= 0) {
            throwInvalidBlockAccessList('storage slots are not sorted');
        }
        prevSlot = slotChange.slot;
        validateSortedUniqueIndices(slotChange.slotChanges.map((change) => Number.parseInt(change.blockAccessIndex, 16)), 'storage');
        if (account.storageReads.includes(slotChange.slot)) {
            throwInvalidBlockAccessList('storage slot appears in both storageChanges and storageReads');
        }
    }
    let prevRead;
    const seenReads = new Set();
    for (const slot of account.storageReads) {
        if (prevRead !== undefined && compareLexicographicHex(prevRead, slot) >= 0) {
            throwInvalidBlockAccessList('storage reads are not sorted');
        }
        if (seenReads.has(slot)) {
            throwInvalidBlockAccessList('duplicate storage read');
        }
        seenReads.add(slot);
        prevRead = slot;
    }
    validateSortedUniqueIndices(account.balanceChanges.map((change) => Number.parseInt(change.blockAccessIndex, 16)), 'balance');
    validateSortedUniqueIndices(account.nonceChanges.map((change) => Number.parseInt(change.blockAccessIndex, 16)), 'nonce');
    validateSortedUniqueIndices(account.codeChanges.map((change) => Number.parseInt(change.blockAccessIndex, 16)), 'code');
}
function validateStorageChanges(storageChanges) {
    let prevSlot;
    for (const [slot, changes] of storageChanges) {
        if (prevSlot !== undefined && compareLexicographicHex(prevSlot, slot) >= 0) {
            throwInvalidBlockAccessList('storage slots are not sorted');
        }
        prevSlot = slot;
        validateSortedUniqueIndices(changes.map(([index]) => index), 'storage');
    }
}
function validateStorageReads(storageReads, storageChanges) {
    const changeSlots = new Set(storageChanges.map(([slot]) => slot));
    let prevRead;
    const seenReads = new Set();
    for (const slot of storageReads) {
        const slotHex = typeof slot === 'string' ? slot : (0, bytes_ts_1.bytesToHex)(slot);
        if (prevRead !== undefined && compareLexicographicHex(prevRead, slotHex) >= 0) {
            throwInvalidBlockAccessList('storage reads are not sorted');
        }
        if (seenReads.has(slotHex)) {
            throwInvalidBlockAccessList('duplicate storage read');
        }
        if (changeSlots.has(slotHex)) {
            throwInvalidBlockAccessList('storage slot appears in both storageChanges and storageReads');
        }
        seenReads.add(slotHex);
        prevRead = slotHex;
    }
}
function validateSortedUniqueIndices(indices, field) {
    let prevIndex;
    const seen = new Set();
    for (const index of indices) {
        if (prevIndex !== undefined && index <= prevIndex) {
            throwInvalidBlockAccessList(`${field} changes are not sorted by block access index`);
        }
        if (seen.has(index)) {
            throwInvalidBlockAccessList(`duplicate block access index in ${field} changes`);
        }
        seen.add(index);
        prevIndex = index;
    }
}
function rawAccountHasChanges(account) {
    const [, storageChanges, storageReads, balanceChanges, nonceChanges, codeChanges] = account;
    return (storageChanges.length > 0 ||
        storageReads.length > 0 ||
        balanceChanges.length > 0 ||
        nonceChanges.length > 0 ||
        codeChanges.length > 0);
}
function accountHasChanges(account) {
    return (account.storageChanges.length > 0 ||
        account.storageReads.length > 0 ||
        account.balanceChanges.length > 0 ||
        account.nonceChanges.length > 0 ||
        account.codeChanges.length > 0);
}
function validateAddress(address) {
    if (address.length !== 42) {
        throwInvalidBlockAccessList(`invalid address length: ${address}`);
    }
    (0, bytes_ts_1.hexToBytes)(address);
}
// Accepts both hex strings and raw bytes because BlockLevelAccessList.raw()
// runs slot/read keys through normalizeHexForRLP, which returns Uint8Array
// for the canonical zero-slot case (an empty bytes encoding). The validator
// sees both shapes when comparing slots that have been normalized for RLP.
function compareLexicographicHex(a, b) {
    const aBytes = a instanceof Uint8Array ? a : (0, bytes_ts_1.hexToBytes)(a);
    const bBytes = b instanceof Uint8Array ? b : (0, bytes_ts_1.hexToBytes)(b);
    const minLength = Math.min(aBytes.length, bBytes.length);
    for (let i = 0; i < minLength; i++) {
        if (aBytes[i] < bBytes[i])
            return -1;
        if (aBytes[i] > bBytes[i])
            return 1;
    }
    if (aBytes.length < bBytes.length)
        return -1;
    if (aBytes.length > bBytes.length)
        return 1;
    return 0;
}
function throwIncorrectBlockFormat(detail) {
    throw (0, errors_ts_1.EthereumJSErrorWithoutCode)(`invalid header: ${detail}`);
}
function throwInvalidBlockAccessList(detail) {
    throw (0, errors_ts_1.EthereumJSErrorWithoutCode)(`invalid block access list: ${detail}`);
}
function throwInvalidBalHash(detail) {
    throw (0, errors_ts_1.EthereumJSErrorWithoutCode)(`invalid block access list hash: ${detail}`);
}
function throwBlockAccessListGasLimitExceeded() {
    throw (0, errors_ts_1.EthereumJSErrorWithoutCode)('block access list gas limit exceeded');
}
//# sourceMappingURL=validation.js.map