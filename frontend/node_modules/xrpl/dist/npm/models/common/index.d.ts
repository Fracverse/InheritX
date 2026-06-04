export declare const RIPPLED_API_V1 = 1;
export declare const RIPPLED_API_V2 = 2;
export declare const DEFAULT_API_VERSION = 2;
export type APIVersion = typeof RIPPLED_API_V1 | typeof RIPPLED_API_V2;
export type LedgerIndex = number | ('validated' | 'closed' | 'current');
export interface XRP {
    currency: 'XRP';
    issuer?: never;
}
export interface IssuedCurrency {
    currency: string;
    issuer: string;
}
export interface MPTCurrency {
    mpt_issuance_id: string;
}
export type Currency = IssuedCurrency | MPTCurrency | XRP;
export interface IssuedCurrencyAmount extends IssuedCurrency {
    value: string;
}
export interface MPTAmount {
    mpt_issuance_id: string;
    value: string;
}
export type Amount = IssuedCurrencyAmount | string;
export type ClawbackAmount = IssuedCurrencyAmount | MPTAmount;
export interface Balance {
    currency: string;
    issuer?: string;
    value: string;
}
export interface Signer {
    Signer: {
        Account: string;
        TxnSignature: string;
        SigningPubKey: string;
    };
}
export interface Memo {
    Memo: {
        MemoData?: string;
        MemoType?: string;
        MemoFormat?: string;
    };
}
export type StreamType = 'consensus' | 'ledger' | 'manifests' | 'peer_status' | 'transactions' | 'transactions_proposed' | 'server' | 'validations';
export interface PathStep {
    account?: string;
    currency?: string;
    issuer?: string;
}
export type Path = PathStep[];
export interface SignerEntry {
    SignerEntry: {
        Account: string;
        SignerWeight: number;
        WalletLocator?: string;
    };
}
export interface ResponseOnlyTxInfo {
    date?: number;
    hash?: string;
    ledger_index?: number;
    ledger_hash?: string;
    inLedger?: number;
}
export interface NFTOffer {
    amount: Amount;
    flags: number;
    nft_offer_index: string;
    owner: string;
    destination?: string;
    expiration?: number;
}
export interface NFToken {
    nft_id: string;
    ledger_index: number;
    owner: string;
    is_burned: boolean;
    flags: number;
    transfer_fee: number;
    issuer: string;
    nft_taxon: number;
    nft_serial: number;
    uri: string;
}
export interface AuthAccount {
    AuthAccount: {
        Account: string;
    };
}
export interface AuthorizeCredential {
    Credential: {
        Issuer: string;
        CredentialType: string;
    };
}
export interface XChainBridge {
    LockingChainDoor: string;
    LockingChainIssue: Currency;
    IssuingChainDoor: string;
    IssuingChainIssue: Currency;
}
export interface PriceData {
    PriceData: {
        BaseAsset: string;
        QuoteAsset: string;
        AssetPrice?: number | string;
        Scale?: number;
    };
}
export interface MPTokenMetadata {
    ticker: string;
    name: string;
    desc?: string;
    icon: string;
    asset_class: string;
    asset_subclass?: string;
    issuer_name: string;
    uris?: MPTokenMetadataUri[];
    additional_info?: string | Record<string, unknown>;
}
export interface MPTokenMetadataUri {
    uri: string;
    category: string;
    title: string;
}
//# sourceMappingURL=index.d.ts.map