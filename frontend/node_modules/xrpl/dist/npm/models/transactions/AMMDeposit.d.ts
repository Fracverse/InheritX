import { Amount, Currency, IssuedCurrencyAmount } from '../common';
import { BaseTransaction, GlobalFlagsInterface } from './common';
export declare enum AMMDepositFlags {
    tfLPToken = 65536,
    tfSingleAsset = 524288,
    tfTwoAsset = 1048576,
    tfOneAssetLPToken = 2097152,
    tfLimitLPToken = 4194304,
    tfTwoAssetIfEmpty = 8388608
}
export interface AMMDepositFlagsInterface extends GlobalFlagsInterface {
    tfLPToken?: boolean;
    tfSingleAsset?: boolean;
    tfTwoAsset?: boolean;
    tfOneAssetLPToken?: boolean;
    tfLimitLPToken?: boolean;
    tfTwoAssetIfEmpty?: boolean;
}
export interface AMMDeposit extends BaseTransaction {
    TransactionType: 'AMMDeposit';
    Asset: Currency;
    Asset2: Currency;
    Amount?: Amount;
    Amount2?: Amount;
    EPrice?: Amount;
    LPTokenOut?: IssuedCurrencyAmount;
}
export declare function validateAMMDeposit(tx: Record<string, unknown>): void;
//# sourceMappingURL=AMMDeposit.d.ts.map