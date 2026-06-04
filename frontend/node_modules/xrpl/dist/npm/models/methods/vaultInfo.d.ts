import { Currency } from '../common';
import { BaseRequest, BaseResponse } from './baseMethod';
export interface VaultInfoRequest extends BaseRequest {
    command: 'vault_info';
    vault_id?: string;
    owner?: string;
    seq?: number;
}
export interface VaultInfoResponse extends BaseResponse {
    result: {
        vault: {
            Account: string;
            Asset: Currency;
            AssetsAvailable?: string;
            AssetsTotal?: string;
            LedgerEntryType: 'Vault';
            Owner: string;
            PreviousTxnID: string;
            PreviousTxnLgrSeq: number;
            Sequence: number;
            index: string;
            shares: {
                Issuer: string;
                LedgerEntryType: string;
                OutstandingAmount: string;
                PreviousTxnID: string;
                PreviousTxnLgrSeq: number;
                Sequence: number;
                index: string;
                OwnerNode?: string;
                mpt_issuance_id?: string;
                DomainID?: string;
                Flags?: number;
            };
            LossUnrealized?: string;
            OwnerNode?: string;
            ShareMPTID?: string;
            WithdrawalPolicy?: number;
            AssetsMaximum?: string;
            Data?: string;
            Scale?: number;
            Flags?: number;
        };
        ledger_hash?: string;
        ledger_index?: number;
        validated?: boolean;
    };
}
//# sourceMappingURL=vaultInfo.d.ts.map