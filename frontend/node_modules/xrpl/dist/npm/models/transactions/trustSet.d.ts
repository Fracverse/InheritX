import { IssuedCurrencyAmount } from '../common';
import { BaseTransaction, GlobalFlagsInterface } from './common';
export declare enum TrustSetFlags {
    tfSetfAuth = 65536,
    tfSetNoRipple = 131072,
    tfClearNoRipple = 262144,
    tfSetFreeze = 1048576,
    tfClearFreeze = 2097152,
    tfSetDeepFreeze = 4194304,
    tfClearDeepFreeze = 8388608
}
export interface TrustSetFlagsInterface extends GlobalFlagsInterface {
    tfSetfAuth?: boolean;
    tfSetNoRipple?: boolean;
    tfClearNoRipple?: boolean;
    tfSetFreeze?: boolean;
    tfClearFreeze?: boolean;
    tfSetDeepFreeze?: boolean;
    tfClearDeepFreeze?: boolean;
}
export interface TrustSet extends BaseTransaction {
    TransactionType: 'TrustSet';
    LimitAmount: IssuedCurrencyAmount;
    QualityIn?: number;
    QualityOut?: number;
    Flags?: number | TrustSetFlagsInterface;
}
export declare function validateTrustSet(tx: Record<string, unknown>): void;
//# sourceMappingURL=trustSet.d.ts.map