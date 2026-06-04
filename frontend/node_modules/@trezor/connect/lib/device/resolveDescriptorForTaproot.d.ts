import { MessagesSchema as Messages } from '@trezor/protobuf';
import { HDNodeResponse } from '../types/api/getPublicKey';
interface ResolveDescriptorForTaprootParams {
    response: HDNodeResponse;
    publicKey: Messages.PublicKey;
}
export declare const resolveDescriptorForTaproot: ({ response, publicKey, }: ResolveDescriptorForTaprootParams) => {
    xpub: string;
    checksum: string;
} | {
    xpub: string;
    checksum: undefined;
};
export {};
//# sourceMappingURL=resolveDescriptorForTaproot.d.ts.map