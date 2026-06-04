export declare const hmacSHA256: (key: Buffer, data: Buffer) => Buffer<ArrayBufferLike>;
export declare const sha256: (buffer: Buffer) => Buffer<ArrayBufferLike>;
export declare const hkdf: (chainingKey: Buffer, input: Buffer) => Buffer<ArrayBufferLike>[];
export declare const hashOfTwo: (hash1: Buffer, hash2: Buffer) => Buffer<ArrayBufferLike>;
export declare const getIvFromNonce: (nonce: number) => Buffer;
export declare const bigEndianBytesToBigInt: (bytes: Uint8Array) => bigint;
export declare const littleEndianBytesToBigInt: (bytes: Uint8Array) => bigint;
//# sourceMappingURL=tools.d.ts.map