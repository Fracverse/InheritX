import { JavaScriptTypeBuilder, Kind, TSchema } from '@sinclair/typebox';
export interface TBuffer extends TSchema {
    [Kind]: 'Buffer';
    static: Buffer;
    type: 'Buffer';
}
export declare class BufferBuilder extends JavaScriptTypeBuilder {
    Buffer(options?: TSchema): TBuffer;
}
//# sourceMappingURL=buffer.d.ts.map