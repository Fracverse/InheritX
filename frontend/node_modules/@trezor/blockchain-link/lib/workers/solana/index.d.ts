import { ClusterUrl, RpcMainnet, RpcSubscriptionsMainnet, SolanaRpcApiMainnet, SolanaRpcSubscriptionsApi } from '@solana/kit';
import type * as MessageTypes from '@trezor/blockchain-link-types/lib/messages';
import { type TokenProgramName } from '@trezor/blockchain-link-utils/lib/solana';
import { BaseWorker } from '../baseWorker';
export type SolanaAPI = Readonly<{
    clusterUrl: ClusterUrl;
    rpc: RpcMainnet<SolanaRpcApiMainnet>;
    rpcSubscriptions: RpcSubscriptionsMainnet<SolanaRpcSubscriptionsApi>;
}>;
export type AccountProgramName = TokenProgramName | 'staking';
declare class SolanaWorker extends BaseWorker<SolanaAPI> {
    protected isConnected(api: SolanaAPI | undefined): api is SolanaAPI;
    private lazyTokens;
    private isTestnet;
    tryConnect(url: string): Promise<SolanaAPI>;
    messageHandler(event: {
        data: MessageTypes.Message;
    }): Promise<true | undefined>;
    disconnect(): void;
}
export default function Solana(): SolanaWorker;
export {};
//# sourceMappingURL=index.d.ts.map