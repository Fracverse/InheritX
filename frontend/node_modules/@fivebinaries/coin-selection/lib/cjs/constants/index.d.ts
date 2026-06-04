import * as CardanoWasm from '@emurgo/cardano-serialization-lib-nodejs';
export declare const CertificateType: {
    readonly STAKE_REGISTRATION: 0;
    readonly STAKE_DEREGISTRATION: 1;
    readonly STAKE_DELEGATION: 2;
    readonly STAKE_POOL_REGISTRATION: 3;
    readonly STAKE_REGISTRATION_CONWAY: 7;
    readonly STAKE_DEREGISTRATION_CONWAY: 8;
    readonly VOTE_DELEGATION: 9;
};
export declare const ERROR: {
    readonly UTXO_BALANCE_INSUFFICIENT: {
        readonly code: "UTXO_BALANCE_INSUFFICIENT";
        readonly message: "UTxO balance insufficient";
    };
    readonly UTXO_VALUE_TOO_SMALL: {
        readonly code: "UTXO_VALUE_TOO_SMALL";
        readonly message: "UTxO value too small";
    };
    readonly UNSUPPORTED_CERTIFICATE_TYPE: {
        readonly code: "UNSUPPORTED_CERTIFICATE_TYPE";
        readonly message: "Unsupported certificate type";
    };
    readonly UTXO_NOT_FRAGMENTED_ENOUGH: {
        readonly code: "UTXO_NOT_FRAGMENTED_ENOUGH";
        readonly message: "UTxO Not fragmented enough.";
    };
};
export declare const CARDANO_PARAMS: {
    readonly PROTOCOL_MAGICS: {
        readonly mainnet: number;
        readonly testnet_preprod: number;
        readonly testnet_preview: number;
    };
    readonly NETWORK_IDS: {
        readonly mainnet: number;
        readonly testnet_preprod: number;
        readonly testnet_preview: number;
    };
    readonly COINS_PER_UTXO_BYTE: "4310";
    readonly MAX_TX_SIZE: 16384;
    readonly MAX_VALUE_SIZE: 5000;
};
export declare const MAX_TOKENS_PER_OUTPUT = 50;
export declare const DATA_COST_PER_UTXO_BYTE: CardanoWasm.DataCost;
