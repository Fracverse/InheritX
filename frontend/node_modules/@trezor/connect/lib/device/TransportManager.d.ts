import { TRANSPORT, Transport } from '@trezor/transport';
import { TypedEmitter } from '@trezor/utils';
type TransportManagerEvents = {
    [TRANSPORT.START]: Transport;
    [TRANSPORT.ERROR]: string;
};
type StartTransport = (transport: Transport, pendingTransportEvent: boolean, signal: AbortSignal) => Promise<void>;
type InitParams = {
    transports: Transport[];
    transportReconnect?: boolean;
    pendingTransportEvent?: boolean;
};
export declare class TransportManager extends TypedEmitter<TransportManagerEvents> {
    private lock;
    private transports;
    private activeTransport?;
    private transportReconnect;
    private upgradeTimeout?;
    private readonly startTransport;
    constructor(startTransport: StartTransport);
    pending(): Promise<void> | undefined;
    get(): Transport | undefined;
    init({ transports, transportReconnect, pendingTransportEvent }: InitParams): Promise<void>;
    dispose(): Promise<void>;
    private selectTransport;
    private scheduleUpgradeCheck;
    private createInitPromise;
}
export {};
//# sourceMappingURL=TransportManager.d.ts.map