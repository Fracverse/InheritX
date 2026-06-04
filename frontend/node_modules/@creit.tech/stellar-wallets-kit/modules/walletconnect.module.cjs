'use strict';

var modal = require('@walletconnect/modal');
var signClient = require('@walletconnect/sign-client');
var types = require('../types.cjs');
var utils = require('../utils.cjs');

const parseWalletConnectSession = (session) => {
    const accounts = session.namespaces.stellar.accounts.map((account) => ({
        network: account.split(':')[1],
        publicKey: account.split(':')[2],
    }));
    return {
        id: session.topic,
        name: session.peer.metadata.name,
        description: session.peer.metadata.description,
        url: session.peer.metadata.url,
        icons: session.peer.metadata.icons[0],
        accounts,
    };
};
const WALLET_CONNECT_ID = 'wallet_connect';
class WalletConnectModule {
    async isAvailable() {
        return true;
    }
    async isPlatformWrapper() {
        const options = [
            {
                provider: 'freighter',
                platform: 'mobile',
            },
        ];
        return !!options.find(({ provider, platform }) => {
            return window.stellar?.provider === provider && window.stellar?.platform === platform;
        });
    }
    constructor(wcParams) {
        this.wcParams = wcParams;
        this.moduleType = types.ModuleType.BRIDGE_WALLET;
        this.productId = WALLET_CONNECT_ID;
        this.productName = 'Wallet Connect';
        this.productUrl = 'https://walletconnect.com/';
        this.productIcon = 'https://stellar.creit.tech/wallet-icons/walletconnect.png';
        if (wcParams.sessionId) {
            this.setSession(wcParams.sessionId);
        }
        if (wcParams.client && wcParams.modal) {
            this.client = wcParams.client;
            this.qrModal = wcParams.modal;
        }
        else {
            signClient.SignClient.init({
                projectId: wcParams.projectId,
                metadata: {
                    name: wcParams.name,
                    url: wcParams.url,
                    description: wcParams.description,
                    icons: wcParams.icons,
                },
            })
                .then(client => {
                console.log('WalletConnect is ready.');
                this.client = client;
                this.qrModal = new modal.WalletConnectModal({ projectId: wcParams.projectId });
                if (wcParams.onSessionDeleted) {
                    this.onSessionDeleted(wcParams.onSessionDeleted);
                }
            })
                .catch(console.error);
        }
    }
    async getAddress() {
        const runChecks = async () => {
            if (!this.client) {
                throw new Error('WalletConnect is not running yet');
            }
        };
        return runChecks()
            .then(async () => {
            const targetSession = await this.getTargetSession();
            return { address: targetSession.accounts[0].publicKey };
        })
            .catch(e => {
            throw utils.parseError(e);
        });
    }
    async signTransaction(xdr, opts) {
        const runChecks = async () => {
            if (!this.client) {
                throw new Error('WalletConnect is not running yet');
            }
        };
        return runChecks()
            .then(async () => {
            const targetSession = await this.getTargetSession({ publicKey: opts?.address });
            const signedTxXdr = await this.client.request({
                topic: targetSession.id,
                chainId: opts?.networkPassphrase === types.WalletNetwork.PUBLIC
                    ? exports.WalletConnectTargetChain.PUBLIC
                    : exports.WalletConnectTargetChain.TESTNET,
                request: {
                    method: this.wcParams.method,
                    params: { xdr },
                },
            }).then((v) => v.signedXDR);
            return { signedTxXdr };
        })
            .catch(e => {
            throw utils.parseError(e);
        });
    }
    async signAuthEntry() {
        throw {
            code: -3,
            message: 'WalletConnect does not support the "signAuthEntry" function',
        };
    }
    async signMessage() {
        throw {
            code: -3,
            message: 'WalletConnect does not support the "signMessage" function',
        };
    }
    async getNetwork() {
        throw {
            code: -3,
            message: 'WalletConnect does not support the "getNetwork" function',
        };
    }
    /**
     * Allows manually setting the current active session to be used in the kit when doing WalletConnect requests
     *
     * @param sessionId The session ID is a placeholder for the session "topic", term used in WalletConnect
     * */
    setSession(sessionId) {
        this.activeSession = sessionId;
    }
    onSessionDeleted(cb) {
        if (!this.client) {
            throw new Error('WalletConnect is not running yet');
        }
        this.client.on('session_delete', data => {
            cb(data.topic);
        });
    }
    async connectWalletConnect() {
        if (!this.client) {
            throw new Error('WalletConnect is not running yet');
        }
        try {
            const { uri, approval } = await this.client.connect({
                requiredNamespaces: {
                    stellar: {
                        methods: [this.wcParams.method],
                        chains: [
                            this.wcParams.network === types.WalletNetwork.PUBLIC
                                ? exports.WalletConnectTargetChain.PUBLIC
                                : exports.WalletConnectTargetChain.TESTNET,
                        ],
                        events: [],
                    },
                },
            });
            const session = await new Promise((resolve, reject) => {
                // Open QRCode modal if a URI was returned (i.e. we're not connecting an existing pairing).
                if (uri) {
                    this.qrModal.openModal({ uri });
                }
                // Await session approval from the wallet.
                approval()
                    .then(session => {
                    this.qrModal.closeModal();
                    resolve(session);
                })
                    .catch(error => {
                    this.qrModal.closeModal();
                    reject(error);
                });
            }).then(parseWalletConnectSession);
            this.setSession(session.id);
            return session;
        }
        catch (e) {
            this.qrModal.closeModal();
            console.error(e);
            throw new Error('There was an error when trying to connect');
        }
    }
    async disconnect() {
        if (!this.client) {
            throw new Error('WalletConnect is not running yet');
        }
        const sessions = await this.getSessions();
        for (const session of sessions) {
            await this.closeSession(session.id);
        }
    }
    async closeSession(sessionId, reason) {
        if (!this.client) {
            throw new Error('WalletConnect is not running yet');
        }
        await this.client.disconnect({
            topic: sessionId,
            reason: {
                message: reason || 'Session closed',
                code: -1,
            },
        });
    }
    async getSessions() {
        if (!this.client) {
            throw new Error('WalletConnect is not running yet');
        }
        return this.client.session.values.map(parseWalletConnectSession);
    }
    async getTargetSession(params) {
        const activeSessions = await this.getSessions();
        let targetSession = activeSessions.find((session) => session.id === this.activeSession || !!session.accounts.find(a => a.publicKey === params?.publicKey));
        if (!targetSession) {
            targetSession = await this.connectWalletConnect();
        }
        return targetSession;
    }
}
exports.WalletConnectTargetChain = void 0;
(function (WalletConnectTargetChain) {
    WalletConnectTargetChain["PUBLIC"] = "stellar:pubnet";
    WalletConnectTargetChain["TESTNET"] = "stellar:testnet";
})(exports.WalletConnectTargetChain || (exports.WalletConnectTargetChain = {}));
exports.WalletConnectAllowedMethods = void 0;
(function (WalletConnectAllowedMethods) {
    WalletConnectAllowedMethods["SIGN"] = "stellar_signXDR";
    WalletConnectAllowedMethods["SIGN_AND_SUBMIT"] = "stellar_signAndSubmitXDR";
})(exports.WalletConnectAllowedMethods || (exports.WalletConnectAllowedMethods = {}));

exports.WALLET_CONNECT_ID = WALLET_CONNECT_ID;
exports.WalletConnectModule = WalletConnectModule;
//# sourceMappingURL=walletconnect.module.cjs.map
