import { Amount, AuthorizeCredential, ClawbackAmount, Currency, IssuedCurrency, IssuedCurrencyAmount, MPTAmount, Memo, Signer, XChainBridge } from '../common';
export declare const MAX_AUTHORIZED_CREDENTIALS = 8;
export declare const VAULT_DATA_MAX_BYTE_LENGTH = 256;
export declare function isRecord(value: unknown): value is Record<string, unknown>;
export declare function isString(str: unknown): str is string;
export declare function isNumber(num: unknown): num is number;
export declare function isNull(inp: unknown): inp is null;
export declare function isValue<V>(value: V): (inp: unknown) => inp is V;
export declare function isXRPLNumber(value: unknown): value is XRPLNumber;
export declare function isCurrency(input: unknown): input is Currency;
export declare function isIssuedCurrency(input: unknown): input is IssuedCurrency;
export declare function isIssuedCurrencyAmount(input: unknown): input is IssuedCurrencyAmount;
export declare function isAuthorizeCredential(input: unknown): input is AuthorizeCredential;
export declare function isMPTAmount(input: unknown): input is MPTAmount;
export declare function isClawbackAmount(input: unknown): input is ClawbackAmount;
export type Account = string;
export type XRPLNumber = string;
export declare function isAccount(account: unknown): account is Account;
export declare function isAmount(amount: unknown): amount is Amount;
export declare function isTokenAmount(amount: unknown): amount is IssuedCurrencyAmount | MPTAmount;
export declare function isXChainBridge(input: unknown): input is XChainBridge;
export declare function isArray<T = unknown>(input: unknown): input is T[];
export declare function isLedgerEntryId(input: unknown): input is string;
export declare function validateHexMetadata(input: unknown, lengthUpto: number): input is string;
export declare function validateRequiredField<T extends Record<string, unknown>, K extends keyof T, V>(tx: T, param: K, checkValidity: (inp: unknown) => inp is V, errorOpts?: {
    txType?: string;
    paramName?: string;
}): asserts tx is T & {
    [P in K]: V;
};
export declare function validateOptionalField<T extends Record<string, unknown>, K extends keyof T, V>(tx: T, param: K, checkValidity: (inp: unknown) => inp is V, errorOpts?: {
    txType?: string;
    paramName?: string;
}): asserts tx is T & {
    [P in K]: V | undefined;
};
export declare enum GlobalFlags {
    tfInnerBatchTxn = 1073741824
}
export interface GlobalFlagsInterface {
    tfInnerBatchTxn?: boolean;
}
export interface BaseTransaction extends Record<string, unknown> {
    Account: Account;
    TransactionType: string;
    Fee?: string;
    Sequence?: number;
    AccountTxnID?: string;
    Flags?: number | GlobalFlagsInterface;
    LastLedgerSequence?: number;
    Memos?: Memo[];
    Signers?: Signer[];
    SourceTag?: number;
    SigningPubKey?: string;
    TicketSequence?: number;
    TxnSignature?: string;
    NetworkID?: number;
    Delegate?: Account;
}
export declare function validateBaseTransaction(common: unknown): asserts common is BaseTransaction;
export declare function parseAmountValue(amount: unknown): number;
export declare function validateCredentialType<T extends BaseTransaction & Record<string, unknown>>(tx: T): void;
export declare function validateCredentialsList(credentials: unknown, transactionType: string, isStringID: boolean, maxCredentials: number): void;
export declare function containsDuplicates(objectList: AuthorizeCredential[] | string[]): boolean;
export declare function isDomainID(domainID: unknown): domainID is string;
//# sourceMappingURL=common.d.ts.map