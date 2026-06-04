import { NodeUsbTransport, Transport, UdpTransport } from '@trezor/transport';
import type { AbstractTransportParams } from '@trezor/transport/lib/transports/abstract';
import { ConnectSettingsTransport } from '../types';
type Params = AbstractTransportParams & {
    sessionsBackgroundUrl?: string | null;
};
export declare const createTransportList: (params: Params) => (existing: Transport[], transports?: ConnectSettingsTransport[]) => (Transport | NodeUsbTransport | UdpTransport)[];
export {};
//# sourceMappingURL=TransportList.d.ts.map