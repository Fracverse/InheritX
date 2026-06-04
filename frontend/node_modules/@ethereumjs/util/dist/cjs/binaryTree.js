"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.generateBinaryTreeChunkSuffixes = exports.getBinaryTreeKeyForStorageSlot = exports.chunkifyBinaryTreeCode = exports.getBinaryTreeKeyForCodeChunk = exports.getBinaryTreeKey = exports.BINARY_TREE_MAIN_STORAGE_OFFSET = exports.BINARY_TREE_NODE_WIDTH = exports.BINARY_TREE_CODE_OFFSET = exports.BINARY_TREE_HEADER_STORAGE_OFFSET = exports.BINARY_TREE_CODE_CHUNK_SIZE = exports.BINARY_TREE_CODE_HASH_LEAF_KEY = exports.BINARY_TREE_BASIC_DATA_LEAF_KEY = exports.BINARY_TREE_BALANCE_BYTES_LENGTH = exports.BINARY_TREE_NONCE_BYTES_LENGTH = exports.BINARY_TREE_CODE_SIZE_BYTES_LENGTH = exports.BINARY_TREE_VERSION_BYTES_LENGTH = exports.BINARY_TREE_BALANCE_OFFSET = exports.BINARY_TREE_NONCE_OFFSET = exports.BINARY_TREE_CODE_SIZE_OFFSET = exports.BINARY_TREE_VERSION_OFFSET = exports.BinaryTreeLeafType = void 0;
exports.getBinaryTreeStem = getBinaryTreeStem;
exports.getBinaryTreeIndicesForStorageSlot = getBinaryTreeIndicesForStorageSlot;
exports.getBinaryTreeIndicesForCodeChunk = getBinaryTreeIndicesForCodeChunk;
exports.decodeBinaryTreeLeafBasicData = decodeBinaryTreeLeafBasicData;
exports.encodeBinaryTreeLeafBasicData = encodeBinaryTreeLeafBasicData;
exports.generateBinaryTreeCodeStems = generateBinaryTreeCodeStems;
const bytes_ts_1 = require("./bytes.js");
/**
 * @dev Returns the 31-bytes binary tree stem for a given address and tree index.
 * @param hashFunction The hashFunction for the binary tree
 * @param {Address} address The address to generate the tree key for.
 * @param treeIndex The index of the tree to generate the key for. Defaults to 0.
 * @return The 31-bytes binary tree stem as a Uint8Array.
 */
function getBinaryTreeStem(hashFunction, address, treeIndex = 0) {
    const address32 = (0, bytes_ts_1.setLengthLeft)(address.toBytes(), 32);
    let treeIndexBytes;
    if (typeof treeIndex === 'number') {
        treeIndexBytes = (0, bytes_ts_1.setLengthRight)((0, bytes_ts_1.int32ToBytes)(Number(treeIndex), true), 32);
    }
    else {
        treeIndexBytes = (0, bytes_ts_1.setLengthRight)((0, bytes_ts_1.bigIntToBytes)(BigInt(treeIndex), true).slice(0, 32), 32);
    }
    const treeStem = hashFunction((0, bytes_ts_1.concatBytes)(address32, treeIndexBytes)).slice(0, 31);
    return treeStem;
}
exports.BinaryTreeLeafType = {
    BasicData: 0,
    CodeHash: 1,
};
exports.BINARY_TREE_VERSION_OFFSET = 0;
exports.BINARY_TREE_CODE_SIZE_OFFSET = 5;
exports.BINARY_TREE_NONCE_OFFSET = 8;
exports.BINARY_TREE_BALANCE_OFFSET = 16;
exports.BINARY_TREE_VERSION_BYTES_LENGTH = 1;
exports.BINARY_TREE_CODE_SIZE_BYTES_LENGTH = 3;
exports.BINARY_TREE_NONCE_BYTES_LENGTH = 8;
exports.BINARY_TREE_BALANCE_BYTES_LENGTH = 16;
exports.BINARY_TREE_BASIC_DATA_LEAF_KEY = (0, bytes_ts_1.intToBytes)(exports.BinaryTreeLeafType.BasicData);
exports.BINARY_TREE_CODE_HASH_LEAF_KEY = (0, bytes_ts_1.intToBytes)(exports.BinaryTreeLeafType.CodeHash);
exports.BINARY_TREE_CODE_CHUNK_SIZE = 31;
exports.BINARY_TREE_HEADER_STORAGE_OFFSET = 64;
exports.BINARY_TREE_CODE_OFFSET = 128;
exports.BINARY_TREE_NODE_WIDTH = 256;
exports.BINARY_TREE_MAIN_STORAGE_OFFSET = BigInt(256) ** BigInt(exports.BINARY_TREE_CODE_CHUNK_SIZE);
/**
 * @dev Returns the tree key for a given binary tree stem, and sub index.
 * @dev Assumes that the tree node width = 256
 * @param stem The 31-bytes binary tree stem as a Uint8Array.
 * @param subIndex The sub index of the tree to generate the key for as a Uint8Array.
 * @return The tree key as a Uint8Array.
 */
const getBinaryTreeKey = (stem, leaf) => {
    switch (leaf) {
        case exports.BinaryTreeLeafType.BasicData:
            return (0, bytes_ts_1.concatBytes)(stem, exports.BINARY_TREE_BASIC_DATA_LEAF_KEY);
        case exports.BinaryTreeLeafType.CodeHash:
            return (0, bytes_ts_1.concatBytes)(stem, exports.BINARY_TREE_CODE_HASH_LEAF_KEY);
        default:
            return (0, bytes_ts_1.concatBytes)(stem, leaf);
    }
};
exports.getBinaryTreeKey = getBinaryTreeKey;
/**
 * Calculates the position of the storage key in the BinaryTree tree, determining
 * both the tree index (the node in the tree) and the subindex (the position within the node).
 * @param {bigint} storageKey - The key representing a specific storage slot.
 * @returns {Object} - An object containing the tree index and subindex
 */
function getBinaryTreeIndicesForStorageSlot(storageKey) {
    let position;
    if (storageKey < exports.BINARY_TREE_CODE_OFFSET - exports.BINARY_TREE_HEADER_STORAGE_OFFSET) {
        position = BigInt(exports.BINARY_TREE_HEADER_STORAGE_OFFSET) + storageKey;
    }
    else {
        position = exports.BINARY_TREE_MAIN_STORAGE_OFFSET + storageKey;
    }
    const treeIndex = position / BigInt(exports.BINARY_TREE_NODE_WIDTH);
    const subIndex = Number(position % BigInt(exports.BINARY_TREE_NODE_WIDTH));
    return { treeIndex, subIndex };
}
/**
 * Calculates the position of the code chunks in the BinaryTree tree, determining
 * both the tree index (the node in the tree) and the subindex (the position within the node).
 * @param {bigint} chunkId - The ID representing a specific chunk.
 * @returns {Object} - An object containing the tree index and subindex
 */
function getBinaryTreeIndicesForCodeChunk(chunkId) {
    const treeIndex = Math.floor((exports.BINARY_TREE_CODE_OFFSET + chunkId) / exports.BINARY_TREE_NODE_WIDTH);
    const subIndex = (exports.BINARY_TREE_CODE_OFFSET + chunkId) % exports.BINARY_TREE_NODE_WIDTH;
    return { treeIndex, subIndex };
}
/**
 * Asynchronously calculates the BinaryTree tree key for the specified code chunk ID.
 * @param {Address} address - The account address to access code for.
 * @param {number} chunkId - The ID of the code chunk to retrieve.
 * @param hashFunction - The hash function used for BinaryTree-related operations.
 * @returns {Uint8Array} - The BinaryTree tree key as a byte array.
 */
const getBinaryTreeKeyForCodeChunk = (address, chunkId, hashFunction) => {
    const { treeIndex, subIndex } = getBinaryTreeIndicesForCodeChunk(chunkId);
    return (0, bytes_ts_1.concatBytes)(getBinaryTreeStem(hashFunction, address, treeIndex), (0, bytes_ts_1.intToBytes)(subIndex));
};
exports.getBinaryTreeKeyForCodeChunk = getBinaryTreeKeyForCodeChunk;
// This code was written by robots based on the reference implementation in EIP-7864
const chunkifyBinaryTreeCode = (code) => {
    const PUSH1 = 0x60; // Assuming PUSH1 is defined as 0x60
    const PUSH32 = 0x7f; // Assuming PUSH32 is defined as 0x7f
    const PUSH_OFFSET = 0x5f; // Assuming PUSH_OFFSET is defined as 0x5f
    // Calculate padding length
    const paddingLength = (31 - (code.length % 31)) % 31;
    const paddedCode = new Uint8Array(code.length + paddingLength);
    paddedCode.set(code);
    // Pre-allocate the bytesToExecData array
    const bytesToExecData = new Uint8Array(paddedCode.length + 32);
    let pos = 0;
    while (pos < paddedCode.length) {
        let pushdataBytes = 0;
        if (PUSH1 <= paddedCode[pos] && paddedCode[pos] <= PUSH32) {
            pushdataBytes = paddedCode[pos] - PUSH_OFFSET;
        }
        pos += 1;
        for (let x = 0; x < pushdataBytes; x++) {
            bytesToExecData[pos + x] = pushdataBytes - x;
        }
        pos += pushdataBytes;
    }
    // Pre-allocate the chunks array
    const numChunks = Math.ceil(paddedCode.length / 31);
    const chunks = new Array(numChunks);
    for (let i = 0, pos = 0; i < numChunks; i++, pos += 31) {
        const chunk = new Uint8Array(32);
        chunk[0] = Math.min(bytesToExecData[pos], 31);
        chunk.set(paddedCode.subarray(pos, pos + 31), 1);
        chunks[i] = chunk;
    }
    return chunks;
};
exports.chunkifyBinaryTreeCode = chunkifyBinaryTreeCode;
/**
 * Asynchronously calculates the BinaryTree tree key for the specified storage slot.
 * @param {Address} address - The account address to access code for.
 * @param {bigint} storageKey - The storage slot key to retrieve the key for.
 * @param hashFunction - The hash function used in the Binary Tree.
 * @returns {Uint8Array} - The BinaryTree tree key as a byte array.
 */
const getBinaryTreeKeyForStorageSlot = (address, storageKey, hashFunction) => {
    const { treeIndex, subIndex } = getBinaryTreeIndicesForStorageSlot(storageKey);
    return (0, bytes_ts_1.concatBytes)(getBinaryTreeStem(hashFunction, address, treeIndex), (0, bytes_ts_1.intToBytes)(subIndex));
};
exports.getBinaryTreeKeyForStorageSlot = getBinaryTreeKeyForStorageSlot;
/**
 * This function extracts and decodes account header elements (version, nonce, code size, and balance)
 * from an encoded `Uint8Array` representation of raw BinaryTree leaf-node basic data. Each component is sliced
 * from the `encodedBasicData` array based on predefined offsets and lengths, and then converted
 * to its appropriate type (integer or BigInt).
 * @param {Uint8Array} encodedBasicData - The encoded BinaryTree leaf basic data containing the version, nonce,
 * code size, and balance in a compact Uint8Array format.
 * @returns {BinaryTreeLeafBasicData} - An object containing the decoded version, nonce, code size, and balance.
 */
function decodeBinaryTreeLeafBasicData(encodedBasicData) {
    const versionBytes = encodedBasicData.slice(0, exports.BINARY_TREE_VERSION_BYTES_LENGTH);
    const nonceBytes = encodedBasicData.slice(exports.BINARY_TREE_NONCE_OFFSET, exports.BINARY_TREE_NONCE_OFFSET + exports.BINARY_TREE_NONCE_BYTES_LENGTH);
    const codeSizeBytes = encodedBasicData.slice(exports.BINARY_TREE_CODE_SIZE_OFFSET, exports.BINARY_TREE_CODE_SIZE_OFFSET + exports.BINARY_TREE_CODE_SIZE_BYTES_LENGTH);
    const balanceBytes = encodedBasicData.slice(exports.BINARY_TREE_BALANCE_OFFSET, exports.BINARY_TREE_BALANCE_OFFSET + exports.BINARY_TREE_BALANCE_BYTES_LENGTH);
    const version = (0, bytes_ts_1.bytesToInt32)(versionBytes);
    const nonce = (0, bytes_ts_1.bytesToBigInt)(nonceBytes);
    const codeSize = (0, bytes_ts_1.bytesToInt32)(codeSizeBytes);
    const balance = (0, bytes_ts_1.bytesToBigInt)(balanceBytes);
    return { version, nonce, codeSize, balance };
}
/**
 * This function takes a `BinaryTreeLeafBasicData` object and encodes its properties
 * (version, nonce, code size, and balance) into a compact `Uint8Array` format. Each
 * property is serialized and padded to match the required byte lengths defined by
 * EIP-7864. Additionally, 4 bytes are reserved for future use as specified
 * in EIP-7864.
 * @param {Account} account - An object containing the version, nonce,
 *   code size, and balance to be encoded.
 * @returns {Uint8Array} - A compact bytes representation of the account header basic data.
 */
function encodeBinaryTreeLeafBasicData(account) {
    const encodedVersion = (0, bytes_ts_1.setLengthLeft)((0, bytes_ts_1.intToBytes)(account.version), exports.BINARY_TREE_VERSION_BYTES_LENGTH);
    // Per EIP-7864, bytes 1-4 are reserved for future use
    const reservedBytes = new Uint8Array([0, 0, 0, 0]);
    const encodedNonce = (0, bytes_ts_1.setLengthLeft)((0, bytes_ts_1.bigIntToBytes)(account.nonce), exports.BINARY_TREE_NONCE_BYTES_LENGTH);
    const encodedCodeSize = (0, bytes_ts_1.setLengthLeft)((0, bytes_ts_1.intToBytes)(account.codeSize), exports.BINARY_TREE_CODE_SIZE_BYTES_LENGTH);
    const encodedBalance = (0, bytes_ts_1.setLengthLeft)((0, bytes_ts_1.bigIntToBytes)(account.balance), exports.BINARY_TREE_BALANCE_BYTES_LENGTH);
    return (0, bytes_ts_1.concatBytes)(encodedVersion, reservedBytes, encodedCodeSize, encodedNonce, encodedBalance);
}
/**
 * Helper method to generate the suffixes for code chunks for putting code
 * @param numChunks number of chunks to generate suffixes for
 * @returns number[] - an array of numbers corresponding to the code chunks being put
 */
const generateBinaryTreeChunkSuffixes = (numChunks) => {
    if (numChunks === 0)
        return [];
    const chunkSuffixes = new Array(numChunks);
    let currentSuffix = exports.BINARY_TREE_CODE_OFFSET;
    for (let x = 0; x < numChunks; x++) {
        chunkSuffixes[x] = currentSuffix;
        currentSuffix++;
        // Reset suffix to 0 if exceeds BINARY_TREE_NODE_WIDTH
        if (currentSuffix >= exports.BINARY_TREE_NODE_WIDTH)
            currentSuffix = 0;
    }
    return chunkSuffixes;
};
exports.generateBinaryTreeChunkSuffixes = generateBinaryTreeChunkSuffixes;
/**
 * Helper method for generating the code stems necessary for putting code
 * @param numChunks the number of code chunks to be put
 * @param address the address of the account getting the code
 * @param hashFunction an initialized {@link BinaryTreeCrypto} object
 * @returns an array of stems for putting code
 */
function generateBinaryTreeCodeStems(numChunks, address, hashFunction) {
    // The maximum number of chunks is 793 (maxCodeSize - 24576) / (bytes per chunk 31) + (round up - 1)
    // Code is stored in chunks starting at leaf index 128 of the leaf node corresponding to the stem of the code's address
    // Code chunks beyond the initial 128 are stored in additional leaf nodes in batches up of up to 256 chunks per leaf node
    // so the maximum number of leaf nodes that can hold contract code for a specific address is 4 leaf nodes (128 chunks in
    // the first leaf node and 256 chunks in up to 3 additional leaf nodes)
    // So, instead of computing every single leaf key (which is a heavy operation), we just compute the stem for the first
    // chunk in each leaf node and can then know that the chunks in between have tree keys in monotonically increasing order
    const numStems = numChunks > exports.BINARY_TREE_CODE_OFFSET ? Math.ceil(numChunks / exports.BINARY_TREE_NODE_WIDTH) + 1 : 1;
    const chunkStems = new Array(numStems);
    // Compute the stem for the initial set of code chunks
    chunkStems[0] = (0, exports.getBinaryTreeKeyForCodeChunk)(address, 0, hashFunction).slice(0, 31);
    for (let stemNum = 0; stemNum < numStems - 1; stemNum++) {
        // Generate additional stems
        const firstChunkKey = (0, exports.getBinaryTreeKeyForCodeChunk)(address, exports.BINARY_TREE_CODE_OFFSET + stemNum * exports.BINARY_TREE_NODE_WIDTH, hashFunction);
        chunkStems[stemNum + 1] = firstChunkKey.slice(0, 31);
    }
    return chunkStems;
}
//# sourceMappingURL=binaryTree.js.map