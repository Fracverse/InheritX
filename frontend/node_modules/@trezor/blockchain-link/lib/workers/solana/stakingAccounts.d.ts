import { Rpc, RpcMainnet, SolanaRpcApiMainnet } from '@solana/kit';
import { StakeStateAccount, StakeStateV2 } from '@solana-program/stake';
import { SolanaStakingAccount } from '@trezor/blockchain-link-types/lib/solana';
export declare const STAKE_ACCOUNT_V2_SIZE = 200;
export declare const FILTER_DATA_SIZE = 200n;
export declare const FILTER_OFFSET = 44n;
export declare const isStake: (state: StakeStateV2) => state is Extract<StakeStateV2, {
    __kind: "Stake";
}>;
export declare const stakeAccountState: (account: StakeStateAccount, currentEpoch: bigint) => string;
export declare const getDelegations: (rpc: RpcMainnet<SolanaRpcApiMainnet> | Rpc<SolanaRpcApiMainnet>, descriptor: string) => Promise<import("@solana/kit").Account<StakeStateAccount, string>[]>;
export declare const getSolanaStakingData: (rpc: RpcMainnet<SolanaRpcApiMainnet>, descriptor: string, epoch: number) => Promise<SolanaStakingAccount[]>;
//# sourceMappingURL=stakingAccounts.d.ts.map