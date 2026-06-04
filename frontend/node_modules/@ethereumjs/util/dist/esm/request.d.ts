import type { PrefixedHexString } from './types.ts';
export type RequestBytes = Uint8Array;
export type CLRequestType = (typeof CLRequestType)[keyof typeof CLRequestType];
export declare const CLRequestType: {
    readonly Deposit: 0;
    readonly Withdrawal: 1;
    readonly Consolidation: 2;
};
export interface RequestJSON {
    type: PrefixedHexString;
    data: PrefixedHexString;
}
export declare class CLRequest<T extends CLRequestType> {
    readonly bytes: Uint8Array;
    get type(): T;
    get data(): Uint8Array<ArrayBufferLike>;
    constructor(requestType: T, requestData: Uint8Array);
}
export declare function createCLRequest(bytes: Uint8Array): CLRequest<CLRequestType>;
//# sourceMappingURL=request.d.ts.map