import { type Client } from '..';
import { Batch, Payment, Transaction } from '../models/transactions';
export declare function txNeedsNetworkID(client: Client): boolean;
export declare function setValidAddresses(tx: Transaction): void;
export declare function setNextValidSequenceNumber(client: Client, tx: Transaction): Promise<void>;
export declare function getTransactionFee(client: Client, tx: Transaction, signersCount?: number): Promise<void>;
export declare function setLatestValidatedLedgerSequence(client: Client, tx: Transaction): Promise<void>;
export declare function checkAccountDeleteBlockers(client: Client, tx: Transaction): Promise<void>;
export declare function handleDeliverMax(tx: Payment): void;
export declare function autofillBatchTxn(client: Client, tx: Batch): Promise<void>;
//# sourceMappingURL=autofill.d.ts.map