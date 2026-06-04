import { Amount, MPTAmount } from '../common';
import { Account, BaseTransaction } from './common';
export interface EscrowCreate extends BaseTransaction {
    TransactionType: 'EscrowCreate';
    Amount: Amount | MPTAmount;
    Destination: Account;
    CancelAfter?: number;
    FinishAfter?: number;
    Condition?: string;
    DestinationTag?: number;
}
export declare function validateEscrowCreate(tx: Record<string, unknown>): void;
//# sourceMappingURL=escrowCreate.d.ts.map