'use client';

import React, { useState } from 'react';
import { X, Upload, FileText, AlertCircle, CheckCircle } from 'lucide-react';
import clsx from 'clsx';

interface DisputeEvidence {
  ipfsHash: string;
  description: string;
  fileName: string;
}

interface DisputeSubmissionModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (dispute: DisputeData) => Promise<void>;
  planId: string;
  beneficiaryAddress: string;
}

interface DisputeData {
  planId: string;
  beneficiaryAddress: string;
  reason: string;
  evidence: DisputeEvidence[];
  description: string;
}

type DisputeReasonType = 'unauthorized_payout' | 'invalid_beneficiary' | 'incorrect_amount' | 'other';

export function DisputeSubmissionModal({
  isOpen,
  onClose,
  onSubmit,
  planId,
  beneficiaryAddress,
}: DisputeSubmissionModalProps) {
  const [step, setStep] = useState<'reason' | 'details' | 'evidence' | 'review' | 'submitted'>('reason');
  const [reason, setReason] = useState<DisputeReasonType>('unauthorized_payout');
  const [description, setDescription] = useState('');
  const [evidenceList, setEvidenceList] = useState<DisputeEvidence[]>([]);
  const [isUploading, setIsUploading] = useState(false);
  const [uploadProgress, setUploadProgress] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const reasonOptions = [
    {
      value: 'unauthorized_payout' as const,
      label: 'Unauthorized Payout',
      description: 'The inheritance payout was made without proper authorization',
    },
    {
      value: 'invalid_beneficiary' as const,
      label: 'Invalid Beneficiary',
      description: 'I am not the correct beneficiary for this plan',
    },
    {
      value: 'incorrect_amount' as const,
      label: 'Incorrect Amount',
      description: 'The payout amount does not match the plan terms',
    },
    {
      value: 'other' as const,
      label: 'Other',
      description: 'Another reason (please describe in details)',
    },
  ];

  const handleFileUpload = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    if (file.size > 10 * 1024 * 1024) {
      setError('File size must be less than 10MB');
      return;
    }

    setIsUploading(true);
    setError(null);

    try {
      for (let i = 0; i <= 100; i += 10) {
        setUploadProgress(i);
        await new Promise(resolve => setTimeout(resolve, 100));
      }

      const ipfsHash = `QmMockHash${Math.random().toString(36).substring(7)}`;

      const newEvidence: DisputeEvidence = {
        ipfsHash,
        fileName: file.name,
        description: '',
      };

      setEvidenceList([...evidenceList, newEvidence]);
      setUploadProgress(0);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Upload failed');
    } finally {
      setIsUploading(false);
    }
  };

  const handleEvidenceDescriptionChange = (index: number, newDescription: string) => {
    const updated = [...evidenceList];
    updated[index].description = newDescription;
    setEvidenceList(updated);
  };

  const removeEvidence = (index: number) => {
    setEvidenceList(evidenceList.filter((_, i) => i !== index));
  };

  const handleSubmit = async () => {
    if (!reason || !description.trim()) {
      setError('Please fill in all required fields');
      return;
    }

    setIsSubmitting(true);
    setError(null);

    try {
      const disputeData: DisputeData = {
        planId,
        beneficiaryAddress,
        reason,
        evidence: evidenceList,
        description,
      };

      await onSubmit(disputeData);
      setStep('submitted');

      setTimeout(() => {
        handleClose();
      }, 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to submit dispute');
      setIsSubmitting(false);
    }
  };

  const handleClose = () => {
    setStep('reason');
    setReason('unauthorized_payout');
    setDescription('');
    setEvidenceList([]);
    setError(null);
    setUploadProgress(0);
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50">
      <div className="relative w-full max-w-2xl rounded-lg bg-white p-6 shadow-lg">
        <button
          onClick={handleClose}
          className="absolute right-4 top-4 text-gray-400 hover:text-gray-600"
        >
          <X className="h-6 w-6" />
        </button>

        {step === 'submitted' && (
          <div className="flex flex-col items-center justify-center py-12">
            <CheckCircle className="h-16 w-16 text-green-600" />
            <h2 className="mt-4 text-2xl font-bold text-gray-900">Dispute Submitted</h2>
            <p className="mt-2 text-center text-gray-600">
              Your dispute has been recorded on-chain.
            </p>
          </div>
        )}

        {step !== 'submitted' && (
          <>
            <h2 className="mb-6 text-2xl font-bold text-gray-900">Submit Inheritance Dispute</h2>

            {error && (
              <div className="mb-4 flex items-start gap-2 rounded-lg bg-red-50 p-3 text-red-800">
                <AlertCircle className="mt-0.5 h-5 w-5 flex-shrink-0" />
                <p className="text-sm">{error}</p>
              </div>
            )}

            {step === 'reason' && (
              <div className="space-y-3">
                <p className="text-sm font-medium text-gray-700">Select dispute reason:</p>
                {reasonOptions.map(option => (
                  <button
                    key={option.value}
                    onClick={() => {
                      setReason(option.value);
                      setStep('details');
                    }}
                    className={clsx(
                      'w-full rounded-lg border-2 p-4 text-left transition-all',
                      reason === option.value
                        ? 'border-blue-500 bg-blue-50'
                        : 'border-gray-200 bg-white hover:border-gray-300'
                    )}
                  >
                    <p className="font-medium text-gray-900">{option.label}</p>
                    <p className="mt-1 text-sm text-gray-600">{option.description}</p>
                  </button>
                ))}
              </div>
            )}

            {step === 'details' && (
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700">
                    Dispute Reason
                  </label>
                  <p className="mt-1 rounded-lg bg-gray-100 p-3 text-sm text-gray-700">
                    {reasonOptions.find(r => r.value === reason)?.label}
                  </p>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700">
                    Description *
                  </label>
                  <textarea
                    value={description}
                    onChange={e => setDescription(e.target.value)}
                    placeholder="Provide detailed explanation..."
                    className="mt-1 h-32 w-full rounded-lg border border-gray-300 p-3 text-sm focus:border-blue-500 focus:outline-none"
                  />
                  <p className="mt-1 text-xs text-gray-500">
                    {description.length}/1000 characters
                  </p>
                </div>

                <div className="flex gap-2">
                  <button
                    onClick={() => setStep('reason')}
                    className="flex-1 rounded-lg border border-gray-300 px-4 py-2 font-medium text-gray-700 hover:bg-gray-50"
                  >
                    Back
                  </button>
                  <button
                    onClick={() => setStep('evidence')}
                    disabled={!description.trim()}
                    className="flex-1 rounded-lg bg-blue-600 px-4 py-2 font-medium text-white disabled:bg-gray-400 hover:bg-blue-700"
                  >
                    Next: Add Evidence
                  </button>
                </div>
              </div>
            )}

            {step === 'evidence' && (
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700">
                    Supporting Evidence (Optional)
                  </label>
                  <p className="mt-1 text-xs text-gray-600">
                    Upload documents or evidence files (Max 10MB each)
                  </p>
                </div>

                <label className="flex cursor-pointer flex-col items-center gap-2 rounded-lg border-2 border-dashed border-gray-300 p-6 transition-colors hover:border-blue-500 hover:bg-blue-50">
                  <Upload className="h-8 w-8 text-gray-400" />
                  <span className="text-sm font-medium text-gray-700">
                    Click to upload or drag and drop
                  </span>
                  <span className="text-xs text-gray-500">PDF, DOC, IMG (up to 10MB)</span>
                  <input
                    type="file"
                    onChange={handleFileUpload}
                    disabled={isUploading}
                    className="hidden"
                  />
                </label>

                {isUploading && (
                  <div className="space-y-2">
                    <div className="h-2 w-full overflow-hidden rounded-full bg-gray-200">
                      <div
                        className="h-full bg-blue-600 transition-all"
                        style={{ width: `${uploadProgress}%` }}
                      />
                    </div>
                    <p className="text-sm text-gray-600">Uploading: {uploadProgress}%</p>
                  </div>
                )}

                {evidenceList.length > 0 && (
                  <div className="space-y-2">
                    <p className="text-sm font-medium text-gray-700">Attached Evidence</p>
                    {evidenceList.map((evidence, index) => (
                      <div key={index} className="rounded-lg border border-gray-200 p-3">
                        <div className="flex items-start gap-2">
                          <FileText className="mt-0.5 h-5 w-5 text-blue-600 flex-shrink-0" />
                          <div className="flex-1 min-w-0">
                            <p className="truncate text-sm font-medium text-gray-900">
                              {evidence.fileName}
                            </p>
                            <p className="text-xs text-gray-500">IPFS: {evidence.ipfsHash.substring(0, 16)}...</p>
                            <input
                              type="text"
                              value={evidence.description}
                              onChange={e =>
                                handleEvidenceDescriptionChange(index, e.target.value)
                              }
                              placeholder="Brief description..."
                              className="mt-2 w-full rounded border border-gray-200 p-2 text-xs focus:border-blue-500 focus:outline-none"
                            />
                          </div>
                          <button
                            onClick={() => removeEvidence(index)}
                            className="flex-shrink-0 text-gray-400 hover:text-red-600"
                          >
                            <X className="h-5 w-5" />
                          </button>
                        </div>
                      </div>
                    ))}
                  </div>
                )}

                <div className="flex gap-2">
                  <button
                    onClick={() => setStep('details')}
                    className="flex-1 rounded-lg border border-gray-300 px-4 py-2 font-medium text-gray-700 hover:bg-gray-50"
                  >
                    Back
                  </button>
                  <button
                    onClick={() => setStep('review')}
                    className="flex-1 rounded-lg bg-blue-600 px-4 py-2 font-medium text-white hover:bg-blue-700"
                  >
                    Review & Submit
                  </button>
                </div>
              </div>
            )}

            {step === 'review' && (
              <div className="space-y-4">
                <div className="space-y-3 rounded-lg bg-gray-50 p-4">
                  <div>
                    <p className="text-xs font-medium text-gray-600">DISPUTE REASON</p>
                    <p className="text-gray-900">
                      {reasonOptions.find(r => r.value === reason)?.label}
                    </p>
                  </div>
                  <div>
                    <p className="text-xs font-medium text-gray-600">DESCRIPTION</p>
                    <p className="text-gray-900">{description}</p>
                  </div>
                  {evidenceList.length > 0 && (
                    <div>
                      <p className="text-xs font-medium text-gray-600">EVIDENCE FILES</p>
                      <ul className="mt-1 list-inside list-disc text-gray-900">
                        {evidenceList.map((e, i) => (
                          <li key={i}>{e.fileName}</li>
                        ))}
                      </ul>
                    </div>
                  )}
                </div>

                <div className="flex gap-2">
                  <button
                    onClick={() => setStep('evidence')}
                    className="flex-1 rounded-lg border border-gray-300 px-4 py-2 font-medium text-gray-700 hover:bg-gray-50"
                  >
                    Back
                  </button>
                  <button
                    onClick={handleSubmit}
                    disabled={isSubmitting}
                    className="flex-1 rounded-lg bg-green-600 px-4 py-2 font-medium text-white disabled:bg-gray-400 hover:bg-green-700"
                  >
                    {isSubmitting ? 'Submitting...' : 'Submit Dispute'}
                  </button>
                </div>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}

export default DisputeSubmissionModal;
