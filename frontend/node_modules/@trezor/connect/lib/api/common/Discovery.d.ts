import { TypedEmitter } from '@trezor/utils';
import { Blockchain } from '../../backend/BlockchainLink';
import type { DeviceCommands } from '../../device/DeviceCommands';
import type { DiscoveryAccount, DiscoveryAccountType } from '../../types';
import type { GetAccountInfo } from '../../types/api/getAccountInfo';
type DiscoveryType = {
    type: DiscoveryAccountType;
    getPath: (index: number) => number[];
};
type GetDescriptor = (path: number[]) => ReturnType<ReturnType<typeof DeviceCommands>['getAccountDescriptor']>;
type DiscoveryOptions = {
    blockchain: Blockchain;
    getDescriptor: GetDescriptor;
    limit?: number;
};
interface Events {
    progress: DiscoveryAccount[];
    complete: void;
}
export declare class Discovery extends TypedEmitter<Events> {
    types: DiscoveryType[];
    private typeIndex;
    accounts: DiscoveryAccount[];
    private coinInfo;
    private blockchain;
    getDescriptor: GetDescriptor;
    private index;
    private interrupted;
    completed: boolean;
    constructor(options: DiscoveryOptions);
    start(details?: GetAccountInfo['details']): Promise<void>;
    stop(): void;
    dispose(): void;
}
export {};
//# sourceMappingURL=Discovery.d.ts.map