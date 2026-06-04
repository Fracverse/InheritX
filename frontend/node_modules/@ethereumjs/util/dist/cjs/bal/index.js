"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.BlockLevelAccessList = void 0;
exports.createBlockLevelAccessList = createBlockLevelAccessList;
exports.createBlockLevelAccessListFromJSON = createBlockLevelAccessListFromJSON;
exports.createBlockLevelAccessListFromRLP = createBlockLevelAccessListFromRLP;
const rlp_1 = require("@ethereumjs/rlp");
const sha3_js_1 = require("@noble/hashes/sha3.js");
const bytes_ts_1 = require("../bytes.js");
const internal_ts_1 = require("../internal.js");
/**
 * In-memory [EIP-7928](https://eips.ethereum.org/EIPS/eip-7928) block access list with
 * canonical RLP/JSON encoding, checkpointing, and mutation helpers used by the VM during execution.
 *
 * @remarks Experimental (Amsterdam): public API and behaviour may change on patch releases.
 * See `@ethereumjs/vm` README section `Amsterdam hardfork (experimental)` for release ↔ spec tracking.
 */
class BlockLevelAccessList {
    constructor(accesses = {}) {
        this.checkpoints = [];
        // Track original (pre-transaction) balances for net-zero detection
        this.originalBalances = new Map();
        // Track original code at the start of each blockAccessIndex for each address
        // Key format: `${address}-${blockAccessIndex}`
        this.originalCodesAtIndex = new Map();
        this.accesses = accesses;
        this.blockAccessIndex = 0;
    }
    /**
     * Canonical RLP encoding of the access list (`RLP.encode(raw())`).
     *
     * @remarks Experimental (Amsterdam): may change on patch releases.
     */
    serialize() {
        return rlp_1.RLP.encode(this.raw());
    }
    /**
     * Header commitment `keccak256(serialize())` used as `blockAccessListHash`.
     *
     * @remarks Experimental (Amsterdam): may change on patch releases.
     */
    hash() {
        return (0, sha3_js_1.keccak_256)(this.serialize());
    }
    checkpoint() {
        this.checkpoints.push({
            accesses: this.cloneAccesses(this.accesses),
            blockAccessIndex: this.blockAccessIndex,
        });
    }
    commit() {
        if (this.checkpoints.length > 0) {
            this.checkpoints.pop();
        }
    }
    revert() {
        const snapshot = this.checkpoints.pop();
        if (!snapshot) {
            return;
        }
        const current = this.accesses;
        this.accesses = snapshot.accesses;
        this.blockAccessIndex = snapshot.blockAccessIndex;
        // Preserve address touches and storage reads across reverts.
        // EIP-7928: When storage writes are reverted, the slot keys MUST still
        // appear in storageReads since the slots were accessed (SSTORE reads
        // the current value for gas calculation).
        for (const [address, access] of Object.entries(current)) {
            if (this.accesses[address] === undefined) {
                // Collect both explicit reads and slots that were written (but will be reverted)
                const allReads = new Set(access.storageReads);
                for (const slot of Object.keys(access.storageChanges)) {
                    allReads.add(slot);
                }
                this.accesses[address] = {
                    nonceChanges: new Map(),
                    balanceChanges: new Map(),
                    codeChanges: [],
                    storageChanges: {},
                    storageReads: allReads,
                };
                continue;
            }
            const target = this.accesses[address];
            // Preserve explicit storageReads
            for (const slot of access.storageReads) {
                target.storageReads.add(slot);
            }
            // EIP-7928: Convert reverted storageChanges to storageReads
            for (const slot of Object.keys(access.storageChanges)) {
                // Only add to reads if not already in the target's storageChanges
                // (a successful write subsumes a read)
                if (target.storageChanges[slot] === undefined) {
                    target.storageReads.add(slot);
                }
            }
        }
    }
    cloneAccesses(accesses) {
        const cloned = {};
        for (const [address, access] of Object.entries(accesses)) {
            const storageChanges = {};
            for (const [slot, changes] of Object.entries(access.storageChanges)) {
                storageChanges[slot] = changes.map(([index, value]) => [index, value]);
            }
            cloned[address] = {
                nonceChanges: new Map(access.nonceChanges),
                balanceChanges: new Map(access.balanceChanges),
                codeChanges: access.codeChanges.map(([index, code]) => [index, code]),
                storageChanges,
                storageReads: new Set(access.storageReads),
            };
        }
        return cloned;
    }
    /**
     * Canonical sorted tuple view used for RLP and validation.
     *
     * @remarks Experimental (Amsterdam): may change on patch releases.
     */
    raw() {
        const bal = [];
        for (const address of Object.keys(this.accesses)
            .sort()
            .filter((address) => shouldIncludeAddress(address, this.accesses[address]))) {
            const data = this.accesses[address];
            // Format storage changes: [slot, [[index, value], ...]]
            // Normalize slot keys for canonical RLP encoding (0 -> empty bytes)
            const storageChanges = Object.entries(data.storageChanges)
                .sort((a, b) => compareLexicographicHexOrBytes(a[0], b[0]))
                .map(([slot, changes]) => [
                normalizeHexForRLP(slot),
                changes
                    .sort((a, b) => a[0] - b[0])
                    .map(([index, value]) => [index, normalizeBytesForRLPQuantity(value)]),
            ]);
            // Normalize storage reads for canonical RLP encoding (0 -> empty bytes)
            const storageReads = Array.from(data.storageReads)
                .map(normalizeHexForRLP)
                .sort((a, b) => compareLexicographicHexOrBytes(a, b));
            const balanceChanges = Array.from(data.balanceChanges.entries())
                .sort(([a], [b]) => a - b)
                .map(([index, balance]) => [index, normalizeQuantityHexForRLP(balance)]);
            const nonceChanges = Array.from(data.nonceChanges.entries())
                .sort(([a], [b]) => a - b)
                .map(([index, nonce]) => [index, normalizeQuantityHexForRLP(nonce)]);
            const codeChanges = [...data.codeChanges].sort(([a], [b]) => a - b);
            bal.push([
                address,
                storageChanges,
                storageReads,
                balanceChanges,
                nonceChanges,
                codeChanges,
            ]);
        }
        return bal;
    }
    addAddress(address) {
        if (this.accesses[address] !== undefined) {
            return;
        }
        this.accesses[address] = {
            storageChanges: {},
            storageReads: new Set(),
            balanceChanges: new Map(),
            nonceChanges: new Map(),
            codeChanges: [],
        };
    }
    addStorageWrite(address, storageKey, value, blockAccessIndex, originalValue) {
        const strippedKey = normalizeStorageKeyHex((0, bytes_ts_1.bytesToHex)(stripLeadingZeros(storageKey)));
        const strippedValue = stripLeadingZeros(value);
        const strippedOriginal = originalValue ? stripLeadingZeros(originalValue) : undefined;
        const isZeroWrite = strippedValue.length === 0;
        // EIP-7928: Check if this is a no-op write (value equals pre-transaction value)
        // No-op writes should be recorded as reads, not changes.
        // Note: Both empty arrays (zero values) compare equal via bytesToHex
        let isNoOp = false;
        if (strippedOriginal !== undefined) {
            // We have original value - compare properly
            isNoOp = (0, bytes_ts_1.bytesToHex)(strippedValue) === (0, bytes_ts_1.bytesToHex)(strippedOriginal);
        }
        else if (isZeroWrite) {
            // No original value provided and writing zero - likely a no-op for system contracts
            // reading empty slots. Treat as read for safety.
            isNoOp = true;
        }
        // Only no-op writes (writing same value as original) are treated as reads
        // EIP-7928: Zeroing a slot (pre-value exists, post-value is zero) IS a write
        if (isNoOp) {
            // EIP-7928: A no-op write within a transaction only affects that tx's
            // blockAccessIndex entry. Prior tx changes for the same slot must remain.
            if (this.accesses[address] !== undefined) {
                const slotChanges = this.accesses[address].storageChanges[strippedKey];
                if (slotChanges !== undefined) {
                    const remaining = slotChanges.filter(([idx]) => idx !== blockAccessIndex);
                    if (remaining.length === 0) {
                        delete this.accesses[address].storageChanges[strippedKey];
                        this.addStorageRead(address, storageKey);
                    }
                    else {
                        this.accesses[address].storageChanges[strippedKey] = remaining;
                    }
                }
                else {
                    this.addStorageRead(address, storageKey);
                }
            }
            else {
                this.addStorageRead(address, storageKey);
            }
            return;
        }
        if (this.accesses[address] === undefined) {
            this.addAddress(address);
        }
        if (this.accesses[address].storageChanges[strippedKey] === undefined) {
            this.accesses[address].storageChanges[strippedKey] = [];
        }
        const slotChanges = this.accesses[address].storageChanges[strippedKey];
        const existingIndex = slotChanges.findIndex(([idx]) => idx === blockAccessIndex);
        if (existingIndex !== -1) {
            slotChanges[existingIndex] = [blockAccessIndex, strippedValue];
        }
        else {
            slotChanges.push([blockAccessIndex, strippedValue]);
        }
        // Per EIP-7928: A successful storage write subsumes any prior read of the same slot.
        // Remove the slot from storageReads since it's now in storageChanges.
        this.accesses[address].storageReads.delete(strippedKey);
    }
    addStorageRead(address, storageKey) {
        if (this.accesses[address] === undefined) {
            this.addAddress(address);
        }
        const strippedKey = normalizeStorageKeyHex((0, bytes_ts_1.bytesToHex)(stripLeadingZeros(storageKey)));
        // Per EIP-7928: Don't add to storageReads if the slot was already written.
        // A write subsumes any reads of the same slot.
        if (this.accesses[address].storageChanges[strippedKey] === undefined) {
            this.accesses[address].storageReads.add(strippedKey);
        }
    }
    addBalanceChange(address, balance, blockAccessIndex, originalBalance) {
        if (this.accesses[address] === undefined) {
            this.addAddress(address);
        }
        // EIP-7928: Track the original (pre-transaction) balance for net-zero detection
        // Only set if not already tracked (first call wins)
        if (originalBalance !== undefined && !this.originalBalances.has(address)) {
            this.originalBalances.set(address, originalBalance);
        }
        this.accesses[address].balanceChanges.set(blockAccessIndex, padToEvenHex((0, bytes_ts_1.bytesToHex)(stripLeadingZeros((0, bytes_ts_1.bigIntToBytes)(balance)))));
    }
    /**
     * EIP-7928: Remove balance changes for addresses where final balance equals first balance.
     * Call this at the end of each transaction to clean up net-zero balance changes.
     */
    cleanupNetZeroBalanceChanges() {
        for (const [address, originalBalance] of this.originalBalances.entries()) {
            const access = this.accesses[address];
            if (access === undefined || access.balanceChanges.size === 0) {
                continue;
            }
            // Get the final balance (last entry in the balanceChanges map)
            const entries = Array.from(access.balanceChanges.values());
            const finalBalanceHex = entries[entries.length - 1];
            const finalBalance = finalBalanceHex === '0x' ? BigInt(0) : BigInt(`0x${finalBalanceHex.replace(/^0x/, '')}`);
            // EIP-7928: If final balance == original balance, remove all balanceChanges
            // for the current blockAccessIndex only, but keep prior tx entries.
            if (finalBalance === originalBalance) {
                access.balanceChanges.delete(this.blockAccessIndex);
            }
        }
        // Clear the tracking map for the next transaction
        this.originalBalances.clear();
    }
    addNonceChange(address, nonce, blockAccessIndex) {
        if (this.accesses[address] === undefined) {
            this.addAddress(address);
        }
        this.accesses[address].nonceChanges.set(blockAccessIndex, padToEvenHex((0, bytes_ts_1.bigIntToHex)(nonce)));
    }
    addCodeChange(address, code, blockAccessIndex, originalCode) {
        if (this.accesses[address] === undefined) {
            this.addAddress(address);
        }
        const codeChanges = this.accesses[address].codeChanges;
        // Track the original code at the start of this blockAccessIndex
        const trackingKey = `${address}-${blockAccessIndex}`;
        if (!this.originalCodesAtIndex.has(trackingKey) && originalCode !== undefined) {
            this.originalCodesAtIndex.set(trackingKey, originalCode);
        }
        // Get the original code at the start of this blockAccessIndex
        const originalCodeAtIndex = this.originalCodesAtIndex.get(trackingKey);
        // Check if there's already a code change at this blockAccessIndex
        const existingIndex = codeChanges.findIndex(([idx]) => idx === blockAccessIndex);
        if (existingIndex !== -1) {
            // Check if the new code equals the original code at start of this blockAccessIndex
            // If so, remove the entry (net-zero change within this blockAccessIndex)
            if (originalCodeAtIndex !== undefined &&
                (0, bytes_ts_1.bytesToHex)(code) === (0, bytes_ts_1.bytesToHex)(originalCodeAtIndex)) {
                codeChanges.splice(existingIndex, 1);
            }
            else {
                // Update the existing entry with the new code
                codeChanges[existingIndex] = [blockAccessIndex, code];
            }
        }
        else {
            // Add new entry, but only if code is actually different from originalCode
            if (originalCode !== undefined && (0, bytes_ts_1.bytesToHex)(code) === (0, bytes_ts_1.bytesToHex)(originalCode)) {
                // No actual change, don't record
                return;
            }
            codeChanges.push([blockAccessIndex, code]);
        }
    }
    /**
     * Converts the internal representation to JSON fixture / Engine API form.
     * Inverse of {@link createBlockLevelAccessListFromJSON}.
     *
     * @remarks Experimental (Amsterdam): may change on patch releases.
     */
    toJSON() {
        const result = [];
        for (const [address, access] of Object.entries(this.accesses)
            .sort(([a], [b]) => a.localeCompare(b))
            .filter(([address, access]) => shouldIncludeAddress(address, access))) {
            const storageChanges = Object.entries(access.storageChanges)
                .sort(([a], [b]) => a.localeCompare(b))
                .map(([slot, changes]) => ({
                slot,
                slotChanges: changes
                    .sort((a, b) => a[0] - b[0])
                    .map(([index, value]) => ({
                    blockAccessIndex: indexToHex(index),
                    postValue: padToEvenHex((0, bytes_ts_1.bytesToHex)(value)),
                })),
            }));
            const storageReads = Array.from(access.storageReads).sort((a, b) => Number((a === '0x' ? 0n : (0, bytes_ts_1.hexToBigInt)(a)) -
                (b === '0x' ? 0n : (0, bytes_ts_1.hexToBigInt)(b))));
            const balanceChanges = Array.from(access.balanceChanges.entries())
                .sort(([a], [b]) => a - b)
                .map(([index, balance]) => ({
                blockAccessIndex: indexToHex(index),
                postBalance: balance,
            }));
            const nonceChanges = Array.from(access.nonceChanges.entries())
                .sort(([a], [b]) => a - b)
                .map(([index, nonce]) => ({
                blockAccessIndex: indexToHex(index),
                postNonce: nonce,
            }));
            const codeChanges = access.codeChanges.map(([index, code]) => ({
                blockAccessIndex: indexToHex(index),
                newCode: (0, bytes_ts_1.bytesToHex)(code),
            }));
            result.push({
                address: address,
                nonceChanges,
                balanceChanges,
                codeChanges,
                storageChanges,
                storageReads,
            });
        }
        return result;
    }
    /**
     * For selfdestructed accounts, drop state changes while preserving read footprints.
     * Any `storageChanges` are converted to `storageReads`. Per EIP-7928, a positive
     * pre-transaction balance reduced to zero via `SELFDESTRUCT` keeps the balance change.
     *
     * @remarks Experimental (Amsterdam): may change on patch releases.
     */
    cleanupSelfdestructed(addresses) {
        for (const address of addresses) {
            const access = this.accesses[address];
            if (access === undefined) {
                continue;
            }
            // Convert any storageChanges into storageReads
            for (const slot of Object.keys(access.storageChanges)) {
                access.storageReads.add(slot);
            }
            access.storageChanges = {};
            access.nonceChanges.clear();
            access.codeChanges = [];
            // EIP-7928: If the account had a positive pre-transaction balance,
            // the balance change to zero MUST be recorded.
            // The balance change to 0 is already added during SELFDESTRUCT execution.
            // We only clear balance changes if pre-transaction balance was 0 (no actual change).
            const originalBalance = this.originalBalances.get(address);
            if (originalBalance === undefined || originalBalance === BigInt(0)) {
                // Pre-transaction balance was 0 or unknown - clear balance changes
                // (0 -> 0 is no change, so nothing to record)
                access.balanceChanges.clear();
            }
            // If originalBalance > 0, keep the balance changes (which should show balance = 0)
        }
    }
}
exports.BlockLevelAccessList = BlockLevelAccessList;
function compareLexicographicHexOrBytes(a, b) {
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
/**
 * Creates an empty {@link BlockLevelAccessList}.
 *
 * @remarks Experimental (Amsterdam): may change on patch releases.
 */
function createBlockLevelAccessList() {
    return new BlockLevelAccessList();
}
/**
 * Parses a JSON block access list (fixture / Engine API order).
 *
 * @remarks Experimental (Amsterdam): may change on patch releases.
 */
function createBlockLevelAccessListFromJSON(json) {
    const bal = new BlockLevelAccessList();
    for (const account of json) {
        bal.addAddress(account.address);
        const access = bal.accesses[account.address];
        for (const slotChange of account.storageChanges) {
            const normalizedSlot = normalizeStorageKeyHex(slotChange.slot);
            if (access.storageChanges[normalizedSlot] === undefined) {
                access.storageChanges[normalizedSlot] = [];
            }
            for (const change of slotChange.slotChanges) {
                access.storageChanges[normalizedSlot].push([
                    parseInt(change.blockAccessIndex, 16),
                    (0, bytes_ts_1.hexToBytes)(change.postValue),
                ]);
            }
        }
        for (const slot of account.storageReads) {
            access.storageReads.add(normalizeStorageKeyHex(slot));
        }
        for (const change of account.balanceChanges) {
            access.balanceChanges.set(parseInt(change.blockAccessIndex, 16), padToEvenHex(change.postBalance));
        }
        for (const change of account.nonceChanges) {
            access.nonceChanges.set(parseInt(change.blockAccessIndex, 16), padToEvenHex(change.postNonce));
        }
        for (const change of account.codeChanges) {
            access.codeChanges.push([parseInt(change.blockAccessIndex, 16), (0, bytes_ts_1.hexToBytes)(change.newCode)]);
        }
    }
    return bal;
}
/**
 * Normalizes a quantity-like hex string for canonical RLP encoding.
 * Integer fields in the BAL use minimal big-endian encoding, so leading zero bytes
 * are stripped and zero is encoded as empty bytes.
 */
function normalizeHexForRLP(hex) {
    const stripped = hex.slice(2).replace(/^0+/, '');
    if (stripped === '') {
        return Uint8Array.from([]);
    }
    return `0x${(0, internal_ts_1.padToEven)(stripped)}`;
}
function normalizeBytesForRLPQuantity(bytes) {
    return stripLeadingZeros(bytes);
}
function normalizeQuantityHexForRLP(hex) {
    const stripped = hex.slice(2).replace(/^0+/, '');
    if (stripped === '') {
        return '0x';
    }
    return `0x${(0, internal_ts_1.padToEven)(stripped)}`;
}
/**
 * Parses an RLP-encoded block access list.
 *
 * @remarks Experimental (Amsterdam): may change on patch releases.
 */
function createBlockLevelAccessListFromRLP(rlp) {
    const decoded = rlp_1.RLP.decode(rlp);
    const bal = new BlockLevelAccessList();
    for (const account of decoded) {
        const [addressBytes, storageChangesRaw, storageReadsRaw, balanceChangesRaw, nonceChangesRaw, codeChangesRaw,] = account;
        const address = (0, bytes_ts_1.bytesToHex)(addressBytes);
        bal.addAddress(address);
        const access = bal.accesses[address];
        for (const [slotBytes, slotChangesRaw] of storageChangesRaw) {
            const slot = normalizeStorageKeyHex((0, bytes_ts_1.bytesToHex)(slotBytes));
            if (access.storageChanges[slot] === undefined) {
                access.storageChanges[slot] = [];
            }
            for (const [indexBytes, valueBytes] of slotChangesRaw) {
                access.storageChanges[slot].push([(0, bytes_ts_1.bytesToInt)(indexBytes), valueBytes]);
            }
        }
        for (const slotBytes of storageReadsRaw) {
            access.storageReads.add(normalizeStorageKeyHex((0, bytes_ts_1.bytesToHex)(slotBytes)));
        }
        for (const [indexBytes, balanceBytes] of balanceChangesRaw) {
            access.balanceChanges.set((0, bytes_ts_1.bytesToInt)(indexBytes), padToEvenHex((0, bytes_ts_1.bytesToHex)(balanceBytes)));
        }
        for (const [indexBytes, nonceBytes] of nonceChangesRaw) {
            access.nonceChanges.set((0, bytes_ts_1.bytesToInt)(indexBytes), padToEvenHex((0, bytes_ts_1.bytesToHex)(nonceBytes)));
        }
        for (const [indexBytes, codeBytes] of codeChangesRaw) {
            access.codeChanges.push([(0, bytes_ts_1.bytesToInt)(indexBytes), codeBytes]);
        }
    }
    return bal;
}
function stripLeadingZeros(bytes) {
    let first = bytes[0];
    while (bytes.length > 0 && first.toString() === '0') {
        bytes = bytes.slice(1);
        first = bytes[0];
    }
    return bytes;
}
function padToEvenHex(hex) {
    return `0x${(0, internal_ts_1.padToEven)(hex.slice(2))}`;
}
/**
 * Normalizes a storage key hex string to ensure consistent even-length representation.
 * - "0x" (empty bytes) is kept as is
 * - "0x0" becomes "0x00"
 * - Any odd-length hex is padded to even (e.g., "0x1" → "0x01")
 */
function normalizeStorageKeyHex(hex) {
    const stripped = hex.slice(2);
    // Empty string "0x" stays as is
    if (stripped === '') {
        return '0x';
    }
    // Pad to even length (handles "0x0" → "0x00", "0x1" → "0x01", etc.)
    return `0x${(0, internal_ts_1.padToEven)(stripped)}`;
}
function shouldIncludeAddress(_address, _access) {
    // All entries in `accesses` were explicitly touched during execution (or supplied via
    // fixture JSON). EIP-7928 requires touched addresses to appear even with empty change
    // lists, including SYSTEM_ADDRESS when accessed directly (e.g. BALANCE).
    return true;
}
function indexToHex(index) {
    return padToEvenHex(`0x${index.toString(16)}`);
}
//# sourceMappingURL=index.js.map