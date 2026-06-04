import { PROTO } from '../constants';
import { AbstractMethod } from '../core/AbstractMethod';
export default class ResetDevice extends AbstractMethod<'resetDevice', PROTO.ResetDevice> {
    init(): void;
    get info(): string;
    get confirmation(): {
        view: "device-management";
        label: string;
    };
    private resetDeviceWorkflow;
    private entropyCheckWorkflow;
    run(): Promise<{
        message: string;
    }>;
}
//# sourceMappingURL=resetDevice.d.ts.map