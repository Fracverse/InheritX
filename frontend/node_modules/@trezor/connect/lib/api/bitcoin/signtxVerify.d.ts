import { Transaction as BitcoinJsTransaction } from '@trezor/utxo-lib';
import { PROTO } from '../../constants';
import type { DeviceCommands } from '../../device/DeviceCommands';
import type { Network } from '../../types';
type GetHDNode = (address_n: number[]) => ReturnType<ReturnType<typeof DeviceCommands>['getHDNode']>;
export declare const deriveOutputScript: (getHDNode: GetHDNode, output: PROTO.TxOutputType, network: Network) => Promise<Buffer<ArrayBufferLike> | undefined>;
export declare const verifyTx: (serializedTx: string, params: {
    inputs: PROTO.TxInputType[];
    outputs: PROTO.TxOutputType[];
    outputScripts: Awaited<ReturnType<typeof deriveOutputScript>>[];
    network: Network;
}) => BitcoinJsTransaction;
export {};
//# sourceMappingURL=signtxVerify.d.ts.map