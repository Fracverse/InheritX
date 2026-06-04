import type { KZG } from './kzg.ts';
import type { PrefixedHexString } from './types.ts';
export declare const CELLS_PER_EXT_BLOB = 128;
/**
 * Converts arbitrary byte data into EIP-4844 blob format.
 * Splits data into 4096 field elements of 32 bytes each, with proper alignment.
 * @param data Input data (must be exactly BLOB_SIZE bytes)
 * @returns Hex-prefixed blob string
 */
export declare function getBlob(data: Uint8Array): PrefixedHexString;
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
export declare const getBlobs: (input: string | string[]) => `0x${string}`[];
/**
 * EIP-4844: Computes KZG commitments for a set of blobs.
 * @param kzg KZG implementation used to compute commitments
 * @param blobs Array of blob data as hex-prefixed strings
 * @returns Array of lowercase hex-prefixed KZG commitments (one per blob)
 */
export declare const blobsToCommitments: (kzg: KZG, blobs: PrefixedHexString[]) => `0x${string}`[];
/**
 * EIP-4844: Computes KZG proofs for each blob/commitment pair.
 * @param kzg KZG implementation used to compute proofs
 * @param blobs Array of blob data as hex-prefixed strings
 * @param commitments Array of corresponding blob commitments
 * @returns Array of lowercase hex-prefixed proofs (aligned with input order)
 */
export declare const blobsToProofs: (kzg: KZG, blobs: PrefixedHexString[], commitments: PrefixedHexString[]) => `0x${string}`[];
/**
 * EIP-4844: Converts a vector commitment for a given data blob to its versioned hash.  For 4844, this version
 * number will be 0x01 for KZG vector commitments but could be different if future vector commitment
 * types are introduced
 * @param commitment a vector commitment to a blob
 * @param blobCommitmentVersion the version number corresponding to the type of vector commitment
 * @returns a versioned hash corresponding to a given blob vector commitment
 */
export declare const computeVersionedHash: (commitment: PrefixedHexString, blobCommitmentVersion: number) => `0x${string}`;
/**
 * EIP-4844: Generate an array of versioned hashes from corresponding kzg commitments
 * @param commitments array of kzg commitments
 * @returns array of versioned hashes
 * Note: assumes KZG commitments (version 1 version hashes)
 */
export declare const commitmentsToVersionedHashes: (commitments: PrefixedHexString[]) => `0x${string}`[];
/**
 * EIP-7594: Expands blobs into their extended cells using the provided KZG implementation.
 * @param kzg KZG implementation capable of computing cells
 * @param blobs Array of blob data as hex-prefixed strings
 * @returns Tuple of [cells, indices], where cells are hex strings and indices are 0..127
 */
export declare const blobsToCells: (kzg: KZG, blobs: PrefixedHexString[]) => [PrefixedHexString[], number[]];
/**
 * EIP-7594: Computes extended cells and corresponding proofs for the given blobs.
 * @param kzg KZG implementation capable of computing cells and proofs
 * @param blobs Array of blob data as hex-prefixed strings
 * @returns Tuple of [cells, proofs, indices]; indices are 0..127
 */
export declare const blobsToCellsAndProofs: (kzg: KZG, blobs: PrefixedHexString[]) => [PrefixedHexString[], PrefixedHexString[], number[]];
/**
 * EIP-7594: Computes cell proofs for the given blobs.
 * @param kzg KZG implementation capable of computing cell proofs
 * @param blobs Array of blob data as hex-prefixed strings
 * @returns Array of lowercase hex-prefixed cell proofs (aligned with input order)
 */
export declare const blobsToCellProofs: (kzg: KZG, blobs: PrefixedHexString[]) => PrefixedHexString[];
//# sourceMappingURL=blobs.d.ts.map