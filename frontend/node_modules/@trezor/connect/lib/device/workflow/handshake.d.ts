import { WorkflowContext } from '../../types/workflow';
import { Log } from '../../utils/debug';
type Context = {
    device: WorkflowContext['device'];
    signal: AbortSignal;
    logger?: Log;
};
export declare const handshakeCancel: ({ device, logger, signal }: Context) => Promise<void>;
export {};
//# sourceMappingURL=handshake.d.ts.map