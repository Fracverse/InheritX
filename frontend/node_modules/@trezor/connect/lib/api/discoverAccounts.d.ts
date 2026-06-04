import { AbstractMethod } from '../core/AbstractMethod';
import type { CoinInfo, FirmwareRange } from '../types';
import { AccountTypeItem, AdditionalParams, CARDANO_DERIVATIONS } from '../types/api/discoverAccounts';
type CardanoDerivation = (typeof CARDANO_DERIVATIONS)[keyof typeof CARDANO_DERIVATIONS];
type Request = AdditionalParams & {
    account: AccountTypeItem;
    derivation?: CardanoDerivation;
    firmwareRange: FirmwareRange;
    coinInfo: CoinInfo;
    offset: number;
    skip: number;
};
export default class DiscoverAccounts extends AbstractMethod<'discoverAccounts', Request[]> {
    disposed: boolean;
    init(): void;
    private progress;
    private updateProgress;
    private sendProgress;
    run(): Promise<{
        empty: number;
        nonempty: number;
        failed: number;
    }>;
    private filterUnsupportedAccounts;
    private filterCardanoDerivations;
    private readonly descriptorLock;
    private readonly descriptorCache;
    private getDescriptor;
    private discoverAccount;
    dispose(): void;
}
export {};
//# sourceMappingURL=discoverAccounts.d.ts.map