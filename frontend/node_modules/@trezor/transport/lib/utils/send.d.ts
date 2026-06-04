import { encodeMessage } from '@trezor/protobuf';
import { ThpState, TransportProtocol } from '@trezor/protocol';
import { AsyncResultWithTypedError } from '../types';
export declare const createChunks: (data: Buffer, chunkHeader: Buffer, chunkSize: number) => Buffer<ArrayBufferLike>[];
interface BuildMessageProps {
    messages: Parameters<typeof encodeMessage>[0];
    name: string;
    data: Record<string, unknown>;
    protocol: TransportProtocol;
    thpState?: ThpState;
}
export declare const buildMessage: ({ messages, name, data, protocol, thpState }: BuildMessageProps) => Buffer<ArrayBufferLike>;
export declare const sendChunks: <T, E>(chunks: Buffer[], apiWrite: (chunk: Buffer) => AsyncResultWithTypedError<T, E>) => Promise<{
    success: false;
    error: E;
    message?: string;
} | {
    success: true;
    payload: undefined;
}>;
export {};
//# sourceMappingURL=send.d.ts.map