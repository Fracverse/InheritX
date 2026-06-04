import type { APIVersion } from '../models';
import { Response, RequestResponseMap } from '../models/methods';
import { BaseRequest, ErrorResponse } from '../models/methods/baseMethod';
export default class RequestManager {
    private nextId;
    private readonly promisesAwaitingResponse;
    addPromise<R extends BaseRequest, T = RequestResponseMap<R, APIVersion>>(newId: string | number, timer: ReturnType<typeof setTimeout>): Promise<T>;
    resolve(id: string | number, response: Partial<Response<APIVersion>>): void;
    reject(id: string | number, error: Error): void;
    rejectAll(error: Error): void;
    createRequest<R extends BaseRequest, T = RequestResponseMap<R, APIVersion>>(request: R, timeout: number): [string | number, string, Promise<T>];
    handleResponse(response: Partial<Response<APIVersion> | ErrorResponse>): void;
    private deletePromise;
}
//# sourceMappingURL=RequestManager.d.ts.map