import { AuthorizeCredential } from '../common';
import { BaseTransaction } from './common';
export interface DepositPreauth extends BaseTransaction {
    TransactionType: 'DepositPreauth';
    Authorize?: string;
    Unauthorize?: string;
    AuthorizeCredentials?: AuthorizeCredential[];
    UnauthorizeCredentials?: AuthorizeCredential[];
}
export declare function validateDepositPreauth(tx: Record<string, unknown>): void;
//# sourceMappingURL=depositPreauth.d.ts.map