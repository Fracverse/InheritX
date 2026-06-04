import { Type } from 'protobufjs/light';
import type { MessageResponse } from './messages';
export declare const decode: (MessageParam: Type, data: Buffer) => {
    [key: string]: any;
};
export declare const decodeMessage: (messages: protobuf.Root, messageType: number | string, data: Buffer) => MessageResponse;
//# sourceMappingURL=decode.d.ts.map