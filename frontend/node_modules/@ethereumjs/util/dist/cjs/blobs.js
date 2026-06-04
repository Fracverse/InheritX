"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.blobsToCellProofs = exports.blobsToCellsAndProofs = exports.blobsToCells = exports.commitmentsToVersionedHashes = exports.computeVersionedHash = exports.blobsToProofs = exports.blobsToCommitments = exports.getBlobs = exports.CELLS_PER_EXT_BLOB = void 0;
exports.getBlob = getBlob;
const sha2_js_1 = require("@noble/hashes/sha2.js");
const bytes_ts_1 = require("./bytes.js");
/**
 * These utilities for constructing blobs are borrowed from https://github.com/Inphi/eip4844-interop.git
 */
const BYTES_PER_FIELD_ELEMENT = 32; // EIP-4844
const FIELD_ELEMENTS_PER_BLOB = 4096; // EIP-4844
const BLOB_SIZE = BYTES_PER_FIELD_ELEMENT * FIELD_ELEMENTS_PER_BLOB;
const MAX_BLOBS_PER_TX = 6; // EIP-7691: Blob throughput increase, Pectra HF
const MAX_BLOB_BYTES_PER_TX = BLOB_SIZE * MAX_BLOBS_PER_TX - 1;
exports.CELLS_PER_EXT_BLOB = 128; // EIP-4844, Consensus Spec, 2 * FIELD_ELEMENTS_PER_BLOB // 64 (FIELD_ELEMENTS_PER_CELL)
/**
 * Pads input data to blob boundaries with 0x80 marker and zeros.
 * @param data Input data to pad
 * @param blobs_len Number of blobs the data should span
 * @returns Padded data aligned to blob boundaries
 */
function getPadded(data, blobs_len) {
    const pData = new Uint8Array(blobs_len * BLOB_SIZE);
    pData.set(data);
    pData[data.byteLength] = 0x80;
    return pData;
}
/**
 * Converts arbitrary byte data into EIP-4844 blob format.
 * Splits data into 4096 field elements of 32 bytes each, with proper alignment.
 * @param data Input data (must be exactly BLOB_SIZE bytes)
 * @returns Hex-prefixed blob string
 */
function getBlob(data) {
    const blob = new Uint8Array(BLOB_SIZE);
    for (let i = 0; i < FIELD_ELEMENTS_PER_BLOB; i++) {
        const chunk = new Uint8Array(32);
        chunk.set(data.subarray(i * 31, (i + 1) * 31), 0);
        blob.set(chunk, i * 32);
    }
    return (0, bytes_ts_1.bytesToHex)(blob);
}
/**
 * EIP-4844: Converts UTF-8 string(s) into EIP-4844 blob format.
 *
 * Each input string is converted to UTF-8 bytes, padded with 0x80 followed by zeros
 * to align with blob boundaries, and encoded as one or more blobs depending on size.
 * Multiple inputs are processed sequentially, with each input contributing its own blob(s).
 *
 * @param input Single UTF-8 string or array of UTF-8 strings to encode
 * @throws Error with message 'invalid blob data' if any input string is empty
 * @throws Error with message 'blob data is too large' if any single input exceeds MAX_USEFUL_BYTES_PER_TX
 * @returns Array of hex-prefixed blob strings (0x...), one blob per 131,071 useful bytes per input
 */
const getBlobs = (input) => {
    const inputArray = Array.isArray(input) ? input : [input];
    const blobs = [];
    for (const input of inputArray) {
        const data = (0, bytes_ts_1.utf8ToBytes)(input);
        const len = data.byteLength;
        if (len === 0) {
            throw Error('invalid blob data (0 bytes)');
        }
        if (len > MAX_BLOB_BYTES_PER_TX) {
            throw Error(`blob data is too large (${len} bytes > ${MAX_BLOB_BYTES_PER_TX} bytes)`);
        }
        const blobs_len = Math.ceil(len / BLOB_SIZE);
        const pData = getPadded(data, blobs_len);
        for (let i = 0; i < blobs_len; i++) {
            const chunk = pData.subarray(i * BLOB_SIZE, (i + 1) * BLOB_SIZE);
            const blob = getBlob(chunk);
            blobs.push(blob);
        }
    }
    return blobs;
};
exports.getBlobs = getBlobs;
/**
 * EIP-4844: Computes KZG commitments for a set of blobs.
 * @param kzg KZG implementation used to compute commitments
 * @param blobs Array of blob data as hex-prefixed strings
 * @returns Array of lowercase hex-prefixed KZG commitments (one per blob)
 */
const blobsToCommitments = (kzg, blobs) => {
    const commitments = [];
    for (const blob of blobs) {
        commitments.push(kzg.blobToKzgCommitment(blob).toLowerCase());
    }
    return commitments;
};
exports.blobsToCommitments = blobsToCommitments;
/**
 * EIP-4844: Computes KZG proofs for each blob/commitment pair.
 * @param kzg KZG implementation used to compute proofs
 * @param blobs Array of blob data as hex-prefixed strings
 * @param commitments Array of corresponding blob commitments
 * @returns Array of lowercase hex-prefixed proofs (aligned with input order)
 */
const blobsToProofs = (kzg, blobs, commitments) => {
    const proofs = blobs.map((blob, ctx) => kzg.computeBlobProof(blob, commitments[ctx]).toLowerCase());
    return proofs;
};
exports.blobsToProofs = blobsToProofs;
/**
 * EIP-4844: Converts a vector commitment for a given data blob to its versioned hash.  For 4844, this version
 * number will be 0x01 for KZG vector commitments but could be different if future vector commitment
 * types are introduced
 * @param commitment a vector commitment to a blob
 * @param blobCommitmentVersion the version number corresponding to the type of vector commitment
 * @returns a versioned hash corresponding to a given blob vector commitment
 */
const computeVersionedHash = (commitment, blobCommitmentVersion) => {
    const computedVersionedHash = new Uint8Array(32);
    computedVersionedHash.set([blobCommitmentVersion], 0);
    computedVersionedHash.set((0, sha2_js_1.sha256)((0, bytes_ts_1.hexToBytes)(commitment)).subarray(1), 1);
    return (0, bytes_ts_1.bytesToHex)(computedVersionedHash);
};
exports.computeVersionedHash = computeVersionedHash;
/**
 * EIP-4844: Generate an array of versioned hashes from corresponding kzg commitments
 * @param commitments array of kzg commitments
 * @returns array of versioned hashes
 * Note: assumes KZG commitments (version 1 version hashes)
 */
const commitmentsToVersionedHashes = (commitments) => {
    const hashes = [];
    for (const commitment of commitments) {
        hashes.push((0, exports.computeVersionedHash)(commitment, 0x01));
    }
    return hashes;
};
exports.commitmentsToVersionedHashes = commitmentsToVersionedHashes;
/**
 * EIP-7594: Expands blobs into their extended cells using the provided KZG implementation.
 * @param kzg KZG implementation capable of computing cells
 * @param blobs Array of blob data as hex-prefixed strings
 * @returns Tuple of [cells, indices], where cells are hex strings and indices are 0..127
 */
const blobsToCells = (kzg, blobs) => {
    const cells = blobs.reduce((acc, elem) => {
        return [...acc, ...kzg.computeCells(elem)];
    }, []);
    const indices = Array.from({ length: exports.CELLS_PER_EXT_BLOB }, (_, i) => i);
    return [cells, indices];
};
exports.blobsToCells = blobsToCells;
/**
 * EIP-7594: Computes extended cells and corresponding proofs for the given blobs.
 * @param kzg KZG implementation capable of computing cells and proofs
 * @param blobs Array of blob data as hex-prefixed strings
 * @returns Tuple of [cells, proofs, indices]; indices are 0..127
 */
const blobsToCellsAndProofs = (kzg, blobs) => {
    const blobsAndCells = blobs.reduce(([cellsAcc, proofsAcc], elem) => {
        const blobCellsAndProofs = kzg.computeCellsAndProofs(elem);
        return [
            [...cellsAcc, ...blobCellsAndProofs[0]],
            [...proofsAcc, ...blobCellsAndProofs[1]],
        ];
    }, [[], []]);
    const indices = Array.from({ length: exports.CELLS_PER_EXT_BLOB }, (_, i) => i);
    return [...blobsAndCells, indices];
};
exports.blobsToCellsAndProofs = blobsToCellsAndProofs;
/**
 * EIP-7594: Computes cell proofs for the given blobs.
 * @param kzg KZG implementation capable of computing cell proofs
 * @param blobs Array of blob data as hex-prefixed strings
 * @returns Array of lowercase hex-prefixed cell proofs (aligned with input order)
 */
const blobsToCellProofs = (kzg, blobs) => {
    return (0, exports.blobsToCellsAndProofs)(kzg, blobs)[1];
};
exports.blobsToCellProofs = blobsToCellProofs;
//# sourceMappingURL=blobs.js.map