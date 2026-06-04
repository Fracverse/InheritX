/**
 * Interface for an externally provided kzg library used when creating blob transactions
 */
export interface KZG {
    blobToKzgCommitment(blob: string): string;
    computeBlobProof(blob: string, commitment: string): string;
    verifyProof(polynomialKZG: string, z: string, y: string, KZGProof: string): boolean;
    verifyBlobProofBatch(blobs: string[], expectedKZGCommitments: string[], KZGProofs: string[]): boolean;
    computeCells(blob: string): string[];
    computeCellsAndProofs(blob: string): [string[], string[]];
    recoverCellsAndProofs(indices: number[], cells: string[]): [string[], string[]];
    verifyCellKzgProofBatch(commitments: string[], indices: number[], cells: string[], proofs: string[]): boolean;
}
//# sourceMappingURL=kzg.d.ts.map