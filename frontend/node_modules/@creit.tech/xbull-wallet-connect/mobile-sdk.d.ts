import { SdkResponse } from './interfaces';
export declare class xBullSDK {
    isConnected: boolean;
    constructor();
    private sendEventToContentScript;
    enableConnection(): Promise<void>;
    getAddress(): Promise<SdkResponse<{
        address: string;
    }>>;
    signTransaction(params: {
        xdr: string;
        opts?: {
            networkPassphrase?: string;
            address?: string;
            submit?: boolean;
            submitUrl?: string;
        };
    }): Promise<SdkResponse<{
        signedTxXdr: string;
        signerAddress: string;
    }>>;
    signMessage(message: string, opts?: {
        networkPassphrase?: string;
        address?: string;
    }): Promise<SdkResponse<{
        signedMessage: string;
        signerAddress: string;
    }>>;
    getNetwork(): Promise<SdkResponse<{
        network: string;
        networkPassphrase: string;
    }>>;
}
