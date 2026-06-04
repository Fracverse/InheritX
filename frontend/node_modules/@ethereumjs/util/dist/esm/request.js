import { concatBytes } from "./bytes.js";
export const CLRequestType = {
    Deposit: 0,
    Withdrawal: 1,
    Consolidation: 2,
};
export class CLRequest {
    get type() {
        return this.bytes[0];
    }
    get data() {
        return this.bytes.subarray(1);
    }
    constructor(requestType, requestData) {
        this.bytes = concatBytes(new Uint8Array([requestType]), requestData);
    }
}
export function createCLRequest(bytes) {
    switch (bytes[0]) {
        case CLRequestType.Deposit:
            return new CLRequest(CLRequestType.Deposit, bytes.subarray(1));
        case CLRequestType.Withdrawal:
            return new CLRequest(CLRequestType.Withdrawal, bytes.subarray(1));
        case CLRequestType.Consolidation:
            return new CLRequest(CLRequestType.Consolidation, bytes.subarray(1));
        default:
            throw Error(`Invalid request type=${bytes[0]}`);
    }
}
//# sourceMappingURL=request.js.map