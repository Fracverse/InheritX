import { MessagesSchema } from '@trezor/protobuf';
interface GetSolanaTokenDefinition {
    mintAddress?: string;
}
export declare const getSolanaTokenDefinition: ({ mintAddress }: GetSolanaTokenDefinition) => Promise<ArrayBuffer | undefined>;
export declare const decodeSolanaTokenDefinition: (encodedDefinition: ArrayBuffer) => MessagesSchema.SolanaTokenInfo;
export {};
//# sourceMappingURL=solanaDefinitions.d.ts.map