import { AuthAccount, Currency, IssuedCurrencyAmount } from '../common';
import { BaseLedgerEntry, HasOptionalPreviousTxnID } from './BaseLedgerEntry';
export interface VoteSlot {
    VoteEntry: {
        Account: string;
        TradingFee: number;
        VoteWeight: number;
    };
}
export default interface AMM extends BaseLedgerEntry, HasOptionalPreviousTxnID {
    LedgerEntryType: 'AMM';
    Account: string;
    Asset: Currency;
    Asset2: Currency;
    AuctionSlot?: {
        Account: string;
        AuthAccounts?: AuthAccount[];
        DiscountedFee: number;
        Expiration: number;
        Price: IssuedCurrencyAmount;
    };
    LPTokenBalance: IssuedCurrencyAmount;
    TradingFee: number;
    VoteSlots?: VoteSlot[];
    Flags: 0;
}
//# sourceMappingURL=AMM.d.ts.map