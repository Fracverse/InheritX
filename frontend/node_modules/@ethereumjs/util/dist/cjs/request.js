"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.CLRequest = exports.CLRequestType = void 0;
exports.createCLRequest = createCLRequest;
const bytes_ts_1 = require("./bytes.js");
exports.CLRequestType = {
    Deposit: 0,
    Withdrawal: 1,
    Consolidation: 2,
};
class CLRequest {
    get type() {
        return this.bytes[0];
    }
    get data() {
        return this.bytes.subarray(1);
    }
    constructor(requestType, requestData) {
        this.bytes = (0, bytes_ts_1.concatBytes)(new Uint8Array([requestType]), requestData);
    }
}
exports.CLRequest = CLRequest;
function createCLRequest(bytes) {
    switch (bytes[0]) {
        case exports.CLRequestType.Deposit:
            return new CLRequest(exports.CLRequestType.Deposit, bytes.subarray(1));
        case exports.CLRequestType.Withdrawal:
            return new CLRequest(exports.CLRequestType.Withdrawal, bytes.subarray(1));
        case exports.CLRequestType.Consolidation:
            return new CLRequest(exports.CLRequestType.Consolidation, bytes.subarray(1));
        default:
            throw Error(`Invalid request type=${bytes[0]}`);
    }
}
//# sourceMappingURL=request.js.map