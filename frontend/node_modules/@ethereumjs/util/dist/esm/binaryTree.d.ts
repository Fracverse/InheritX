import type { Account } from './account.ts';
import type { Address } from './address.ts';
import type { PrefixedHexString } from './types.ts';
/**
 * @dev Returns the 31-bytes binary tree stem for a given address and tree index.
 * @param hashFunction The hashFunction for the binary tree
 * @param {Address} address The address to generate the tree key for.
 * @param treeIndex The index of the tree to generate the key for. Defaults to 0.
 * @return The 31-bytes binary tree stem as a Uint8Array.
 */
export declare function getBinaryTreeStem(hashFunction: (value: Uint8Array) => Uint8Array, address: Address, treeIndex?: number | bigint): Uint8Array;
export interface BinaryTreeStateDiff {
    stem: PrefixedHexString;
    suffixDiffs: {
        currentValue: PrefixedHexString | null;
        newValue: PrefixedHexString | null;
        suffix: number | string;
    }[];
}
export type BinaryTreeProof = any;
/**
 * Experimental, object format could eventual change.
 * An object that provides the state and proof necessary for binary tree stateless execution
 * */
export interface BinaryTreeExecutionWitness {
    /**
     * The stateRoot of the parent block
     */
    parentStateRoot: PrefixedHexString;
    /**
     * An array of state diffs.
     * Each item corresponding to state accesses or state modifications of the block.
     * In the current design, it also contains the resulting state of the block execution (post-state).
     */
    stateDiff: BinaryTreeStateDiff[];
    /**
     * The proof for the block.
     * Proves that the provided stateDiff belongs to the canonical binary tree.
     */
    proof: BinaryTreeProof;
}
export type BinaryTreeLeafType = (typeof BinaryTreeLeafType)[keyof typeof BinaryTreeLeafType];
export declare const BinaryTreeLeafType: {
    readonly BasicData: 0;
    readonly CodeHash: 1;
};
export type BinaryTreeLeafBasicData = {
    version: number;
    nonce: bigint;
    balance: bigint;
    codeSize: number;
};
export declare const BINARY_TREE_VERSION_OFFSET = 0;
export declare const BINARY_TREE_CODE_SIZE_OFFSET = 5;
export declare const BINARY_TREE_NONCE_OFFSET = 8;
export declare const BINARY_TREE_BALANCE_OFFSET = 16;
export declare const BINARY_TREE_VERSION_BYTES_LENGTH = 1;
export declare const BINARY_TREE_CODE_SIZE_BYTES_LENGTH = 3;
export declare const BINARY_TREE_NONCE_BYTES_LENGTH = 8;
export declare const BINARY_TREE_BALANCE_BYTES_LENGTH = 16;
export declare const BINARY_TREE_BASIC_DATA_LEAF_KEY: Uint8Array<ArrayBufferLike>;
export declare const BINARY_TREE_CODE_HASH_LEAF_KEY: Uint8Array<ArrayBufferLike>;
export declare const BINARY_TREE_CODE_CHUNK_SIZE = 31;
export declare const BINARY_TREE_HEADER_STORAGE_OFFSET = 64;
export declare const BINARY_TREE_CODE_OFFSET = 128;
export declare const BINARY_TREE_NODE_WIDTH = 256;
export declare const BINARY_TREE_MAIN_STORAGE_OFFSET: bigint;
/**
 * @dev Returns the tree key for a given binary tree stem, and sub index.
 * @dev Assumes that the tree node width = 256
 * @param stem The 31-bytes binary tree stem as a Uint8Array.
 * @param subIndex The sub index of the tree to generate the key for as a Uint8Array.
 * @return The tree key as a Uint8Array.
 */
export declare const getBinaryTreeKey: (stem: Uint8Array, leaf: BinaryTreeLeafType | Uint8Array) => Uint8Array<ArrayBuffer>;
/**
 * Calculates the position of the storage key in the BinaryTree tree, determining
 * both the tree index (the node in the tree) and the subindex (the position within the node).
 * @param {bigint} storageKey - The key representing a specific storage slot.
 * @returns {Object} - An object containing the tree index and subindex
 */
export declare function getBinaryTreeIndicesForStorageSlot(storageKey: bigint): {
    treeIndex: bigint;
    subIndex: number;
};
/**
 * Calculates the position of the code chunks in the BinaryTree tree, determining
 * both the tree index (the node in the tree) and the subindex (the position within the node).
 * @param {bigint} chunkId - The ID representing a specific chunk.
 * @returns {Object} - An object containing the tree index and subindex
 */
export declare function getBinaryTreeIndicesForCodeChunk(chunkId: number): {
    treeIndex: number;
    subIndex: number;
};
/**
 * Asynchronously calculates the BinaryTree tree key for the specified code chunk ID.
 * @param {Address} address - The account address to access code for.
 * @param {number} chunkId - The ID of the code chunk to retrieve.
 * @param hashFunction - The hash function used for BinaryTree-related operations.
 * @returns {Uint8Array} - The BinaryTree tree key as a byte array.
 */
export declare const getBinaryTreeKeyForCodeChunk: (address: Address, chunkId: number, hashFunction: (input: Uint8Array) => Uint8Array) => Uint8Array<ArrayBuffer>;
export declare const chunkifyBinaryTreeCode: (code: Uint8Array) => Uint8Array<ArrayBufferLike>[];
/**
 * Asynchronously calculates the BinaryTree tree key for the specified storage slot.
 * @param {Address} address - The account address to access code for.
 * @param {bigint} storageKey - The storage slot key to retrieve the key for.
 * @param hashFunction - The hash function used in the Binary Tree.
 * @returns {Uint8Array} - The BinaryTree tree key as a byte array.
 */
export declare const getBinaryTreeKeyForStorageSlot: (address: Address, storageKey: bigint, hashFunction: (input: Uint8Array) => Uint8Array) => Uint8Array<ArrayBuffer>;
/**
 * This function extracts and decodes account header elements (version, nonce, code size, and balance)
 * from an encoded `Uint8Array` representation of raw BinaryTree leaf-node basic data. Each component is sliced
 * from the `encodedBasicData` array based on predefined offsets and lengths, and then converted
 * to its appropriate type (integer or BigInt).
 * @param {Uint8Array} encodedBasicData - The encoded BinaryTree leaf basic data containing the version, nonce,
 * code size, and balance in a compact Uint8Array format.
 * @returns {BinaryTreeLeafBasicData} - An object containing the decoded version, nonce, code size, and balance.
 */
export declare function decodeBinaryTreeLeafBasicData(encodedBasicData: Uint8Array): BinaryTreeLeafBasicData;
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
export declare function encodeBinaryTreeLeafBasicData(account: Account): Uint8Array;
/**
 * Helper method to generate the suffixes for code chunks for putting code
 * @param numChunks number of chunks to generate suffixes for
 * @returns number[] - an array of numbers corresponding to the code chunks being put
 */
export declare const generateBinaryTreeChunkSuffixes: (numChunks: number) => number[];
/**
 * Helper method for generating the code stems necessary for putting code
 * @param numChunks the number of code chunks to be put
 * @param address the address of the account getting the code
 * @param hashFunction an initialized {@link BinaryTreeCrypto} object
 * @returns an array of stems for putting code
 */
export declare function generateBinaryTreeCodeStems(numChunks: number, address: Address, hashFunction: (input: Uint8Array) => Uint8Array): Uint8Array[];
//# sourceMappingURL=binaryTree.d.ts.map