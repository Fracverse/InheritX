import { IButtonTheme, ISupportedWallet, IModalTheme, KitActions, ModuleInterface, WalletNetwork } from './types';
export interface StellarWalletsKitParams {
    selectedWalletId?: string;
    network: WalletNetwork;
    modules: ModuleInterface[];
    /**
     * @deprecated - This parameter will be removed in a future release, use `modalTheme` instead
     */
    theme?: IModalTheme;
    modalTheme?: IModalTheme;
    buttonTheme?: IButtonTheme;
}
export declare class StellarWalletsKit implements KitActions {
    private buttonElement?;
    private modalElement?;
    private readonly modules;
    private get selectedModule();
    constructor(params: StellarWalletsKitParams);
    /**
     * This method will return an array with all wallets supported by this kit but will let you know those the user have already installed/has access to
     * There are wallets that are by default available since they either don't need to be installed or have a fallback
     */
    getSupportedWallets(): Promise<ISupportedWallet[]>;
    setWallet(id: string): void;
    getAddress(params?: {
        path?: string;
        skipRequestAccess?: boolean;
    }): Promise<{
        address: string;
    }>;
    signTransaction(xdr: string, opts?: {
        networkPassphrase?: string;
        address?: string;
        path?: string;
        submit?: boolean;
        submitUrl?: string;
    }): Promise<{
        signedTxXdr: string;
        signerAddress?: string;
    }>;
    signAuthEntry(authEntry: string, opts?: {
        networkPassphrase?: string;
        address?: string;
        path?: string;
    }): Promise<{
        signedAuthEntry: string;
        signerAddress?: string;
    }>;
    signMessage(message: string, opts?: {
        networkPassphrase?: string;
        address?: string;
        path?: string;
    }): Promise<{
        signedMessage: string;
        signerAddress?: string;
    }>;
    getNetwork(): Promise<{
        network: string;
        networkPassphrase: string;
    }>;
    disconnect(): Promise<void>;
    isButtonCreated(): boolean;
    /**
     * This method allows developers to set their own buttons (for connection and disconnection) on their website
     * while letting the kit handle the logic behind opening the modal, setting and removing the address from the storage, etc
     */
    assignButtons(params: {
        connectEl: HTMLElement | string;
        disconnectEl?: HTMLElement | string;
        onConnect: (response: {
            address: string;
        }) => void;
        onDisconnect: () => void;
    }): void;
    /**
     *
     * @param params {Object}
     * @param params.container {HTMLElement} - The container where the button should be rendered.
     * @param params.onConnect {Function} - This callback is called after the user has clicked the button and selected a wallet
     * @param params.onClosed {Function} - This callback is called if the user closes the modal without selecting any wallet.
     * @param params.onError {Function} - This callback is called if there is an error while trying to get the address once the user has selected the wallet from the modal.
     * @param params.onDisconnect {Function} - This callback is called once the user disconnects from the dropdown modal
     * @param params.horizonUrl {String} - If this url is set, the dropdown modal will show the current XLM balance of the address fetched from the wallet
     * @param params.buttonText {String} - A custom text to set inside the button.
     */
    createButton(params: {
        container: HTMLElement;
        onConnect: (response: {
            address: string;
        }) => void;
        onClosed?: (err: Error) => void;
        onError?: (err: Error) => void;
        onDisconnect: () => void;
        horizonUrl?: string;
        buttonText?: string;
    }): Promise<void>;
    /**
     * Removes the button elements from the HTML and from the kit's instance.
     *
     * @param params.skipDisconnect - Set this to `true` if you want to prevent that we disconnect (for example, disconnecting WalletConnect or removing the address)
     */
    removeButton(params?: {
        skipDisconnect?: boolean;
    }): Promise<void>;
    openModal(params: {
        onWalletSelected: (option: ISupportedWallet) => void;
        onClosed?: (err: Error) => void;
        modalTitle?: string;
        notAvailableText?: string;
    }): Promise<void>;
}
//# sourceMappingURL=stellar-wallets-kit.d.ts.map