"use client";

import { useRef, useState } from "react";
import { Upload, FileCheck, AlertCircle } from "lucide-react";
import {
  createGeneticVerificationAPI,
  GeneticUploadResult,
} from "@/app/lib/api/geneticVerification";
import VerificationStatusCard from "./VerificationStatusCard";

interface Props {
  planId: string;
  onUploadComplete?: (result: GeneticUploadResult) => void;
}

const ACCEPTED_EXTENSIONS = [".txt", ".csv", ".vcf"];
const MAX_FILE_SIZE = 50 * 1024 * 1024; // raw DNA export files run large

export default function GeneticUploadComponent({
  planId,
  onUploadComplete,
}: Props) {
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [file, setFile] = useState<File | null>(null);
  const [isDragOver, setIsDragOver] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<GeneticUploadResult | null>(null);

  const api = createGeneticVerificationAPI();

  const validateAndSetFile = (selected: File) => {
    const hasValidExtension = ACCEPTED_EXTENSIONS.some((ext) =>
      selected.name.toLowerCase().endsWith(ext),
    );
    if (!hasValidExtension) {
      setError(
        `Unsupported file type. Accepted formats: ${ACCEPTED_EXTENSIONS.join(", ")}`,
      );
      return;
    }
    if (selected.size > MAX_FILE_SIZE) {
      setError("File size must be less than 50MB");
      return;
    }
    setFile(selected);
    setError(null);
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selected = e.target.files?.[0];
    if (selected) validateAndSetFile(selected);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);
    const dropped = e.dataTransfer.files?.[0];
    if (dropped) validateAndSetFile(dropped);
  };

  const handleUpload = async () => {
    if (!planId) {
      setError("Missing plan ID. Open this page from one of your plans.");
      return;
    }
    if (!file) {
      setError("Please select a genetic data file");
      return;
    }

    setLoading(true);
    setError(null);
    setResult(null);

    try {
      const dnaHash = await api.computeFileHash(file);
      const uploadResult = await api.submitGeneticHash(planId, dnaHash);
      setResult(uploadResult);
      onUploadComplete?.(uploadResult);
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Upload failed. Please try again.",
      );
    } finally {
      setLoading(false);
    }
  };

  const handleReset = () => {
    setFile(null);
    setError(null);
    setResult(null);
  };

  return (
    <div className="space-y-6">
      <div className="rounded-2xl border border-[#1C252A] bg-[#0D1417] p-6">
        <h2 className="mb-2 text-xl font-semibold text-white">
          Upload Genetic Data
        </h2>
        <p className="mb-6 text-sm text-[#92A5A8]">
          Accepted exports: 23andMe, AncestryDNA, MyHeritage, FamilyTreeDNA,
          or a lab-issued VCF file.
        </p>

        <div
          onClick={() => fileInputRef.current?.click()}
          onDragOver={(e) => {
            e.preventDefault();
            setIsDragOver(true);
          }}
          onDragLeave={() => setIsDragOver(false)}
          onDrop={handleDrop}
          className={`cursor-pointer rounded-xl border-2 border-dashed p-8 text-center transition-colors ${
            isDragOver
              ? "border-[#33C5E0] bg-[#33C5E0]/10"
              : error
                ? "border-[#F56565]"
                : "border-[#2A3338] hover:border-[#33C5E0]/50"
          }`}
        >
          <input
            ref={fileInputRef}
            type="file"
            accept={ACCEPTED_EXTENSIONS.join(",")}
            onChange={handleFileChange}
            className="hidden"
            disabled={loading}
          />
          {file ? (
            <div className="flex items-center justify-center gap-2 text-[#33C5E0]">
              <FileCheck size={20} />
              <span className="font-medium">
                {file.name} ({(file.size / 1024).toFixed(1)} KB)
              </span>
            </div>
          ) : (
            <>
              <Upload className="mx-auto mb-2 text-[#33C5E0]" size={24} />
              <p className="text-sm text-white">
                Click to upload or drag and drop
              </p>
              <p className="mt-1 text-xs text-[#6B7C7F]">
                {ACCEPTED_EXTENSIONS.join(", ")} up to 50MB
              </p>
            </>
          )}
        </div>

        <div className="mt-4 flex items-start gap-2 rounded-lg border border-[#1C252A] bg-[#0A0F11] p-4 text-xs text-[#92A5A8]">
          <AlertCircle
            size={16}
            className="mt-0.5 flex-shrink-0 text-[#33C5E0]"
          />
          <p>
            Your file is hashed in your browser using the Web Crypto API. The
            raw genetic data never leaves your device — only the resulting
            fingerprint is submitted for verification.
          </p>
        </div>

        {error && (
          <div className="mt-4 rounded-lg border border-[#F56565]/30 bg-[#F56565]/10 p-3 text-sm text-[#F56565]">
            {error}
          </div>
        )}

        <div className="mt-6 flex gap-4">
          <button
            onClick={handleUpload}
            disabled={loading || !file}
            className="flex-1 rounded-lg bg-[#33C5E0] px-6 py-3 font-semibold text-[#0D1417] transition-colors hover:bg-[#2AB8D3] disabled:cursor-not-allowed disabled:opacity-50"
          >
            {loading ? "Hashing & submitting…" : "Submit for Verification"}
          </button>
          {(file || error) && (
            <button
              onClick={handleReset}
              className="rounded-lg border border-[#2A3338] px-6 py-3 font-medium text-[#92A5A8] transition-colors hover:bg-[#1C252A]"
            >
              Reset
            </button>
          )}
        </div>
      </div>

      {result && (
        <VerificationStatusCard status={result.status} dnaHash={result.dnaHash} />
      )}
    </div>
  );
}