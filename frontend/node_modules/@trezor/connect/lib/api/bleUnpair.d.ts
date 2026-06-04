import { PROTO } from '../constants';
import { AbstractMethod } from '../core/AbstractMethod';
export default class BleUnpair extends AbstractMethod<'bleUnpair', PROTO.BleUnpair> {
    init(): void;
    run(): Promise<{
        message: string;
    }>;
}
//# sourceMappingURL=bleUnpair.d.ts.map