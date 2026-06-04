import type { TransactionStream, TransactionV1Stream } from '..';
import type { APIVersion } from '../models/common';
import type { RequestResponseMap } from '../models/methods';
import { BaseRequest } from '../models/methods/baseMethod';
export declare function handlePartialPayment<R extends BaseRequest, T = RequestResponseMap<R, APIVersion>>(command: string, response: T): void;
export declare function handleStreamPartialPayment(stream: TransactionStream | TransactionV1Stream, log: (id: string, message: string) => void): void;
//# sourceMappingURL=partialPayment.d.ts.map