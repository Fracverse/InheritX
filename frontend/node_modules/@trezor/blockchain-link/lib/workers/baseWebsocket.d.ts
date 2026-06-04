import { WebsocketClient, WebsocketRequest, WebsocketResponse } from '@trezor/websocket-client';
export declare abstract class BaseWebsocket<T extends Record<string, any>> extends WebsocketClient<T> {
    private readonly subscriptions;
    onPing(): Promise<void>;
    protected onMessage(message: WebsocketResponse): void;
    sendMessage(message: WebsocketRequest): Promise<any>;
    protected addSubscription<E extends keyof T>(type: E, callback: (result: T[E]) => void): void;
    protected removeSubscription(type: keyof T): number;
}
//# sourceMappingURL=baseWebsocket.d.ts.map