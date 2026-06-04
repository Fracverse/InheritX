import type { TypedCall } from '../../device/DeviceCommands';
import { StellarTransaction } from '../../types/api/stellar';
export declare const stellarSignTx: (typedCall: TypedCall, address_n: number[], networkPassphrase: string, tx: StellarTransaction) => Promise<{
    public_key: string;
    signature: string;
}>;
//# sourceMappingURL=stellarSignTx.d.ts.map