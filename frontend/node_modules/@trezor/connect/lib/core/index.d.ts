import EventEmitter from 'events';
import { AbstractMethod } from './AbstractMethod';
import { CoreEventMessage, CoreRequestMessage, TransportInfo } from '../events';
import type { ConnectSettings } from '../types';
import { LogWriter } from '../utils/debug';
export declare class Core extends EventEmitter {
    private abortController;
    private callMethods;
    private popupPromise;
    private methodSynchronize;
    private uiPromises;
    private waitForFirstMethod;
    private _interactionTimeout?;
    private get interactionTimeout();
    private _deviceList?;
    private get deviceList();
    private sendCoreMessage;
    private getCoreContext;
    handleMessage(message: CoreRequestMessage): void;
    dispose(): void;
    getCurrentMethod(): Promise<AbstractMethod<any, undefined>>;
    getActiveTransports(): TransportInfo[] | undefined;
    enumerate(): void;
    init(settings: ConnectSettings, onCoreEvent: (message: CoreEventMessage) => void, logWriterFactory?: () => LogWriter | undefined): Promise<void>;
}
export declare const initCoreState: () => {
    get: () => Core | undefined;
    getPending: () => Promise<Core> | undefined;
    getOrInit: (settings: ConnectSettings, onCoreEvent: (message: CoreEventMessage) => void, logWriterFactory?: (() => LogWriter | undefined) | undefined) => Promise<Core>;
    dispose: () => void;
};
//# sourceMappingURL=index.d.ts.map