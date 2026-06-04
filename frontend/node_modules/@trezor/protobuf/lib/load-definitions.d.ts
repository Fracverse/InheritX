import { Root } from 'protobufjs/light';
type Definitions = Record<string, unknown>;
export declare const loadDefinitions: (messages: Root, packageName: string, packageLoader: () => Definitions | Promise<Definitions>) => Promise<void>;
export {};
//# sourceMappingURL=load-definitions.d.ts.map