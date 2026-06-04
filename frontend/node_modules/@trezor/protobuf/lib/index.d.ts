import * as protobuf from 'protobufjs/light';
import { parseConfigure as parse } from './utils';
export declare const parseConfigure: typeof parse, decodeMessage: (messages: protobuf.Root, messageType: number | string, data: Buffer) => import("./messages").MessageResponse, encodeMessage: (messages: protobuf.Root, messageName: string, data: Record<string, unknown>) => {
    messageType: number;
    message: Buffer<ArrayBuffer>;
};
export * as Messages from './messages';
export { loadDefinitions } from './load-definitions';
export * as MessagesSchema from './messages-schema';
//# sourceMappingURL=index.d.ts.map