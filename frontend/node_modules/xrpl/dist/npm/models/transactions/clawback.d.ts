import { ClawbackAmount } from '../common';
import { BaseTransaction } from './common';
export interface Clawback extends BaseTransaction {
    TransactionType: 'Clawback';
    Account: string;
    Amount: ClawbackAmount;
    Holder?: string;
}
export declare function validateClawback(tx: Record<string, unknown>): void;
//# sourceMappingURL=clawback.d.ts.map