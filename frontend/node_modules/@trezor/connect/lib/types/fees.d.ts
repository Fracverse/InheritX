import { Static } from '@trezor/schema-utils';
export type FeeInfo = Static<typeof FeeInfo>;
export declare const FeeInfo: import("@trezor/schema-utils").TObject<{
    blockTime: import("@trezor/schema-utils").TNumber;
    minFee: import("@trezor/schema-utils").TNumber;
    maxFee: import("@trezor/schema-utils").TNumber;
    minPriorityFee: import("@trezor/schema-utils").TNumber;
    dustLimit: import("@trezor/schema-utils").TNumber;
}>;
export type FeeLevel = Static<typeof FeeLevel>;
export declare const FeeLevel: import("@trezor/schema-utils").TObject<{
    label: import("@trezor/schema-utils").TUnion<[import("@trezor/schema-utils").TLiteral<"high">, import("@trezor/schema-utils").TLiteral<"normal">, import("@trezor/schema-utils").TLiteral<"economy">, import("@trezor/schema-utils").TLiteral<"low">, import("@trezor/schema-utils").TLiteral<"custom">]>;
    blocks: import("@trezor/schema-utils").TNumber;
    feePerUnit: import("@trezor/schema-utils").TString;
    feePerTx: import("@trezor/schema-utils").TOptional<import("@trezor/schema-utils").TString>;
    feeLimit: import("@trezor/schema-utils").TOptional<import("@trezor/schema-utils").TString>;
    baseFeePerGas: import("@trezor/schema-utils").TOptional<import("@trezor/schema-utils").TString>;
    maxFeePerGas: import("@trezor/schema-utils").TOptional<import("@trezor/schema-utils").TString>;
    maxPriorityFeePerGas: import("@trezor/schema-utils").TOptional<import("@trezor/schema-utils").TString>;
}>;
export type SelectFeeLevel = Static<typeof SelectFeeLevel>;
export declare const SelectFeeLevel: import("@trezor/schema-utils").TUnion<[import("@trezor/schema-utils").TObject<{
    name: import("@trezor/schema-utils").TString;
    fee: import("@trezor/schema-utils").TLiteral<"0">;
    feePerByte: import("@trezor/schema-utils").TOptional<import("@trezor/schema-utils").TUndefined>;
    blocks: import("@trezor/schema-utils").TOptional<import("@trezor/schema-utils").TUndefined>;
    disabled: import("@trezor/schema-utils").TLiteral<true>;
}>, import("@trezor/schema-utils").TObject<{
    name: import("@trezor/schema-utils").TString;
    fee: import("@trezor/schema-utils").TString;
    feePerByte: import("@trezor/schema-utils").TString;
    blocks: import("@trezor/schema-utils").TNumber;
    minutes: import("@trezor/schema-utils").TNumber;
    total: import("@trezor/schema-utils").TString;
}>]>;
//# sourceMappingURL=fees.d.ts.map