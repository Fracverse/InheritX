import { secp256k1 } from '@noble/curves/secp256k1.js';
import { Address } from './address.ts';
import type { EOACode7702AuthorizationListBytesItem, EOACode7702AuthorizationListBytesItemUnsigned, EOACode7702AuthorizationListItem, EOACode7702AuthorizationListItemUnsigned } from './types.ts';
export declare const EOA_CODE_7702_AUTHORITY_SIGNING_MAGIC: Uint8Array<ArrayBufferLike>;
/**
 * Converts an authorization list to a JSON format
 * @param authorizationList
 * @returns authorizationList in JSON format
 */
export declare function eoaCode7702AuthorizationListBytesItemToJSON(authorizationList: EOACode7702AuthorizationListBytesItem): EOACode7702AuthorizationListItem;
/**
 * Converts an authority list in JSON to a bytes format
 * @param authorizationList
 * @returns bytes format of the authority list
 */
export declare function eoaCode7702AuthorizationListJSONItemToBytes(authorizationList: EOACode7702AuthorizationListItem): EOACode7702AuthorizationListBytesItem;
/**
 * Returns the bytes (RLP-encoded) to sign
 * @param input Either the bytes or the object format of the authorization list item
 * @returns
 */
export declare function eoaCode7702AuthorizationMessageToSign(input: EOACode7702AuthorizationListItemUnsigned | EOACode7702AuthorizationListBytesItemUnsigned): Uint8Array<ArrayBuffer>;
/**
 * Hashes the RLP-encoded message to sign
 * @param input
 * @returns
 */
export declare function eoaCode7702AuthorizationHashedMessageToSign(input: EOACode7702AuthorizationListItemUnsigned | EOACode7702AuthorizationListBytesItemUnsigned): Uint8Array<ArrayBufferLike>;
/**
 * Signs an authorization list item and returns it in `bytes` format.
 * To get the JSON format, use `authorizationListBytesToJSON([signed])[0] to convert it`
 * @param input
 * @param privateKey
 * @param ecSign
 * @returns
 */
export declare function eoaCode7702SignAuthorization(input: EOACode7702AuthorizationListItemUnsigned | EOACode7702AuthorizationListBytesItemUnsigned, privateKey: Uint8Array, ecSign?: (msg: Uint8Array, pk: Uint8Array, ecSignOpts?: {
    extraEntropy?: Uint8Array | boolean;
}) => Pick<ReturnType<typeof secp256k1.Signature.fromBytes>, 'recovery' | 'r' | 's'>): EOACode7702AuthorizationListBytesItem;
export declare function eoaCode7702RecoverAuthority(input: EOACode7702AuthorizationListItem | EOACode7702AuthorizationListBytesItem): Address;
//# sourceMappingURL=authorization.d.ts.map