import type { BlockchainEventMessage } from './blockchain';
import type { IFrameCallMessage, MethodResponseMessage } from './call';
import type { DeviceEventMessage } from './device';
import type { IFrameEventMessage, IFrameInit, IFrameLogRequest } from './iframe';
import type { PopupAnalyticsResponse, PopupClosedMessage, PopupEventMessage } from './popup';
import type { TransportDisableWebUSB, TransportEventMessage, TransportGetInfo, TransportRequestWebUSBDevice, TransportSetTransports } from './transport';
import type { UiEventMessage } from './ui-request';
import type { UiResponseEvent } from './ui-response';
import { ErrorCode, TrezorError } from '../constants/errors';
import type { Unsuccessful } from '../types/params';
export declare const CORE_EVENT = "CORE_EVENT";
export type CoreRequestMessage = PopupClosedMessage | PopupAnalyticsResponse | TransportDisableWebUSB | TransportSetTransports | TransportRequestWebUSBDevice | TransportGetInfo | UiResponseEvent | IFrameInit | IFrameCallMessage | IFrameLogRequest;
export type CoreEventMessage = {
    success?: boolean;
    channel?: {
        here: string;
        peer: string;
    };
} & (BlockchainEventMessage | DeviceEventMessage | TransportEventMessage | UiEventMessage | MethodResponseMessage | IFrameEventMessage | PopupEventMessage);
export declare const parseMessage: <T extends CoreRequestMessage | CoreEventMessage = never>(messageData: any) => T;
export declare const createErrorMessage: (error: (Error & {
    code?: ErrorCode;
}) | TrezorError) => Unsuccessful;
//# sourceMappingURL=core.d.ts.map