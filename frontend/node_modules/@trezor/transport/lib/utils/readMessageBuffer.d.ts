import { ABORTED_BY_SIGNAL, INTERFACE_DATA_TRANSFER } from '../errors';
import { ResultWithTypedError } from '../types';
type ReadResult = ResultWithTypedError<Buffer, typeof ABORTED_BY_SIGNAL | typeof INTERFACE_DATA_TRANSFER>;
export declare const readMessageBuffer: () => {
    onMessage: (path: string, data: Buffer) => void;
    read: (path: string, signal?: AbortSignal) => Promise<ReadResult>;
    cancelRead: (path: string) => void;
};
export {};
//# sourceMappingURL=readMessageBuffer.d.ts.map