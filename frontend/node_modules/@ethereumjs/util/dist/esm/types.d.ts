import type { Address } from './address.ts';
import type { ToBytesInputTypes } from './bytes.ts';
export type BigIntLike = bigint | PrefixedHexString | number | Uint8Array;
export type BytesLike = Uint8Array | number[] | number | bigint | TransformableToBytes | PrefixedHexString;
export type NumericString = `${number}`;
export type PrefixedHexString = `0x${string}`;
/**
 * A type that represents an input that can be converted to an Address.
 */
export type AddressLike = Address | Uint8Array | PrefixedHexString;
export interface TransformableToBytes {
    toBytes?(): Uint8Array;
}
export type NestedUint8Array = Array<Uint8Array | NestedUint8Array>;
export declare function isNestedUint8Array(value: unknown): value is NestedUint8Array;
export type TypeOutput = (typeof TypeOutput)[keyof typeof TypeOutput];
export declare const TypeOutput: {
    readonly Number: 0;
    readonly BigInt: 1;
    readonly Uint8Array: 2;
    readonly PrefixedHexString: 3;
};
export type TypeOutputReturnType = {
    [TypeOutput.Number]: number;
    [TypeOutput.BigInt]: bigint;
    [TypeOutput.Uint8Array]: Uint8Array;
    [TypeOutput.PrefixedHexString]: PrefixedHexString;
};
/**
 * Convert an input to a specified type.
 * Input of null/undefined returns null/undefined regardless of the output type.
 * @param input value to convert
 * @param outputType type to output
 */
export declare function toType<T extends TypeOutput>(input: null, outputType: T): null;
export declare function toType<T extends TypeOutput>(input: undefined, outputType: T): undefined;
export declare function toType<T extends TypeOutput>(input: ToBytesInputTypes, outputType: T): TypeOutputReturnType[T];
/**
 * EIP-7702 Authorization list types
 */
export type EOACode7702AuthorizationListItemUnsigned = {
    chainId: PrefixedHexString;
    address: PrefixedHexString;
    nonce: PrefixedHexString;
};
export type EOACode7702AuthorizationListItem = {
    yParity: PrefixedHexString;
    r: PrefixedHexString;
    s: PrefixedHexString;
} & EOACode7702AuthorizationListItemUnsigned;
export type EOACode7702AuthorizationListBytesItem = [
    Uint8Array,
    Uint8Array,
    Uint8Array,
    Uint8Array,
    Uint8Array,
    Uint8Array
];
export type EOACode7702AuthorizationListBytes = EOACode7702AuthorizationListBytesItem[];
export type EOACode7702AuthorizationList = EOACode7702AuthorizationListItem[];
export type EOACode7702AuthorizationListBytesItemUnsigned = [Uint8Array, Uint8Array, Uint8Array];
export declare function isEOACode7702AuthorizationListBytes(input: EOACode7702AuthorizationListBytes | EOACode7702AuthorizationList): input is EOACode7702AuthorizationListBytes;
export declare function isEOACode7702AuthorizationList(input: EOACode7702AuthorizationListBytes | EOACode7702AuthorizationList): input is EOACode7702AuthorizationList;
//# sourceMappingURL=types.d.ts.map