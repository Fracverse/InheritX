export declare const aesgcm: (key: Buffer, iv: Buffer) => {
    auth: (authData: Buffer) => void;
    encrypt: (plainText: Buffer) => Buffer<ArrayBuffer>;
    decrypt: (cipherText: Buffer, authTag: Buffer) => Buffer<ArrayBuffer>;
    finish: () => Buffer<ArrayBufferLike>;
};
//# sourceMappingURL=aesgcm.d.ts.map