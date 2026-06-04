import { PROTO } from '../../constants';
export declare const generateEntropy: (len: number) => Buffer<ArrayBuffer>;
type VerifyEntropyOptions = {
    type?: PROTO.Enum_BackupType;
    strength?: number;
    commitment?: string;
    hostEntropy: string;
    trezorEntropy?: string;
    xpubs: Record<string, string>;
};
export declare const verifyEntropy: ({ type, strength, trezorEntropy, hostEntropy, commitment, xpubs, }: VerifyEntropyOptions) => Promise<{
    success: true;
    error?: undefined;
} | {
    success: false;
    error: any;
}>;
export {};
//# sourceMappingURL=verifyEntropy.d.ts.map