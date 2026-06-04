import { PROTO } from '../../../constants';
import { AbstractMethod } from '../../../core/AbstractMethod';
import type { EthereumNetworkInfoDefinitionValues } from '../../../types';
type Params = PROTO.EthereumGetAddress & {
    address?: string;
    network?: EthereumNetworkInfoDefinitionValues;
    encoded_network?: ArrayBuffer;
};
export default class EthereumGetAddress extends AbstractMethod<'ethereumGetAddress', Params[]> {
    hasBundle?: boolean;
    progress: number;
    init(): void;
    initAsync(): Promise<void>;
    get info(): string;
    getButtonRequestData(code: string): {
        type: "address";
        serializedPath: string;
        address: string;
    } | undefined;
    get confirmation(): {
        view: "export-address";
        label: string;
    };
    private _call;
    run(): Promise<import("../../../types").Address | import("../../../types").Address[]>;
}
export {};
//# sourceMappingURL=ethereumGetAddress.d.ts.map