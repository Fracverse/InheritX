import * as protobuf from 'protobufjs/light';
import type { MessageKey } from './messages';
export declare const isPrimitiveField: (field: any) => boolean;
export declare function parseConfigure(data: protobuf.INamespace): protobuf.Root;
export declare const createMessageFromName: (messages: protobuf.Root, name: string) => {
    Message: protobuf.Type;
    messageType: number;
};
export declare const createMessageFromType: (messages: protobuf.Root, messageType: number | string) => {
    Message: protobuf.Type;
    messageName: MessageKey;
};
//# sourceMappingURL=utils.d.ts.map