import type { AccessList, AccessListBytes } from '../types.ts';
/**
 * Converts an access list in bytes to a JSON format
 * @param accessList
 * @returns JSON format of the access list
 */
export declare function accessListBytesToJSON(accessList: AccessListBytes): AccessList;
/**
 * Converts an access list in JSON to a bytes format
 * @param accessList
 * @returns bytes format of the access list
 */
export declare function accessListJSONToBytes(accessList: AccessList): AccessListBytes;
//# sourceMappingURL=access.d.ts.map