import { NETWORKS } from "@shared/constants/stellar";
export type AssetsListKey = NETWORKS.PUBLIC | NETWORKS.TESTNET;
export type AssetsLists = {
    [K in AssetsListKey]: AssetsListItem[];
};
export interface AssetsListItem {
    url: string;
    isEnabled: boolean;
}
export declare const DEFAULT_ASSETS_LISTS: AssetsLists;
export interface AssetListReponseItem {
    code: string;
    issuer: string;
    contract: string;
    org?: string;
    domain: string;
    icon: string;
    decimals: number;
    name?: string;
}
export interface AssetListResponse {
    name: string;
    description: string;
    network: string;
    version: string;
    provider: string;
    assets: AssetListReponseItem[];
}
