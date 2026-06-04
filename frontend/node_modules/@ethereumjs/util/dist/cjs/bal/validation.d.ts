import { type BALJSONBlockAccessList, type BlockLevelAccessList } from './index.ts';
/**
 * EIP-7928 gas cost attributed to each BAL item (one address or one storage key).
 *
 * @remarks Experimental (Amsterdam): may change on patch releases.
 */
export declare const BLOCK_ACCESS_LIST_ITEM_COST = 2000;
/**
 * Counts BAL items per EIP-7928: `addresses + storage_keys`.
 * Uses the canonical {@link BlockLevelAccessList.raw()} view.
 *
 * @remarks Experimental (Amsterdam): may change on patch releases.
 */
export declare function countBlockAccessListItems(bal: BlockLevelAccessList): number;
/**
 * Ensures `bal_items <= block_gas_limit // ITEM_COST` (EIP-7928).
 *
 * @throws if the access list exceeds the block gas-derived item cap
 * @remarks Experimental (Amsterdam): may change on patch releases.
 */
export declare function validateBlockAccessListGasLimit(bal: BlockLevelAccessList, blockGasLimit: bigint): void;
/**
 * Validates canonical BAL structure and, when provided, the header hash commitment.
 *
 * @remarks Experimental (Amsterdam): may change on patch releases.
 */
export declare function validateBlockAccessList(bal: BlockLevelAccessList, blockAccessListHash?: Uint8Array): void;
/**
 * Validates lexicographic ordering, uniqueness, and field constraints of a
 * block access list in JSON fixture / Engine API form.
 *
 * Use before {@link createBlockLevelAccessListFromJSON} so out-of-order or
 * duplicate accounts are not silently merged.
 *
 * @remarks Experimental (Amsterdam): may change on patch releases.
 */
export declare function validateBlockAccessListJSONStructure(json: BALJSONBlockAccessList): void;
/**
 * True when accounts are out of lexicographic order in a way that indicates a full reorder
 * (e.g. `reverse_accounts()`), as opposed to a single appended account breaking sort at the end.
 *
 * @remarks Experimental (Amsterdam): may change on patch releases.
 */
export declare function isAccountOrderOnlyViolation(json: BALJSONBlockAccessList): boolean;
/**
 * `keccak256(rlp(bal))` using the JSON account order (not re-sorted), matching Engine API bytes.
 *
 * @remarks Experimental (Amsterdam): may change on patch releases.
 */
export declare function hashBlockAccessListFromJSON(json: BALJSONBlockAccessList): Uint8Array;
/**
 * Verifies the header hash against the JSON-ordered RLP encoding.
 *
 * @remarks Experimental (Amsterdam): may change on patch releases.
 */
export declare function validateBlockAccessListHashFromJSON(json: BALJSONBlockAccessList, expectedHash: Uint8Array): void;
/**
 * Validates the canonical RLP-oriented structure of a {@link BlockLevelAccessList}.
 *
 * @remarks Experimental (Amsterdam): may change on patch releases.
 */
export declare function validateBlockAccessListStructure(bal: BlockLevelAccessList): void;
/**
 * Verifies `keccak256(rlp(bal))` matches the committed header hash.
 *
 * @remarks Experimental (Amsterdam): may change on patch releases.
 */
export declare function validateBlockAccessListHash(bal: BlockLevelAccessList, expectedHash: Uint8Array): void;
/**
 * Returns true when two access lists produce identical canonical RLP.
 *
 * @remarks Experimental (Amsterdam): may change on patch releases.
 */
export declare function equalsBlockAccessList(a: BlockLevelAccessList, b: BlockLevelAccessList): boolean;
//# sourceMappingURL=validation.d.ts.map