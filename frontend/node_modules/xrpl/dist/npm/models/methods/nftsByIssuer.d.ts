import { NFToken } from '../common';
import { BaseRequest, BaseResponse, LookupByLedgerRequest } from './baseMethod';
export interface NFTsByIssuerRequest extends BaseRequest, LookupByLedgerRequest {
    command: 'nfts_by_issuer';
    issuer: string;
    marker?: unknown;
    nft_taxon?: number;
    limit?: number;
}
export interface NFTsByIssuerResponse extends BaseResponse {
    result: {
        issuer: string;
        nfts: NFToken[];
        marker?: unknown;
        limit?: number;
        nft_taxon?: number;
    };
}
//# sourceMappingURL=nftsByIssuer.d.ts.map