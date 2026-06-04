import type { MPTokenMetadata } from '../common';
export declare const MAX_MPT_META_BYTE_LENGTH = 1024;
export declare const MPT_META_WARNING_HEADER: string;
export declare function encodeMPTokenMetadata(mptokenMetadata: MPTokenMetadata): string;
export declare function decodeMPTokenMetadata(input: string): MPTokenMetadata;
export declare function validateMPTokenMetadata(input: string): string[];
//# sourceMappingURL=mptokenMetadata.d.ts.map