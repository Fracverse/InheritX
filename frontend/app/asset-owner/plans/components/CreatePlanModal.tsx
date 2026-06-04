"use client";

import React, { useState } from "react";
import { X, Plus, Trash2, Check, Loader } from "lucide-react";

interface CreatePlanModalProps {
    onClose: () => void;
}

type Step = "details" | "beneficiaries" | "review" | "approve" | "create";

const STEPS: { key: Step; label: string }[] = [
    { key: "details", label: "Details" },
    { key: "beneficiaries", label: "Beneficiaries" },
    { key: "review", label: "Review" },
    { key: "approve", label: "Approve" },
    { key: "create", label: "Create" },
];

// Mock beneficiaries data
const initialBeneficiaries = [
    { name: "", email: "", relationship: "", allocatedPercentage: 100 },
];

export default function CreatePlanModal({ onClose }: CreatePlanModalProps) {
    const [step, setStep] = useState<Step>("details");
    const [beneficiaries, setBeneficiaries] = useState(initialBeneficiaries);

    // Form state (static, doesn't actually do anything)
    const [planName, setPlanName] = useState("");
    const [planDescription, setPlanDescription] = useState("");
    const [assetType, setAssetType] = useState("ERC20_TOKEN1");
    const [assetAmount, setAssetAmount] = useState("");
    const [distributionMethod, setDistributionMethod] = useState("LUMP_SUM");
    const [transferDate, setTransferDate] = useState("");

    const currentStepIndex = STEPS.findIndex((s) => s.key === step);

    const addBeneficiary = () => {
        if (beneficiaries.length < 10) {
            setBeneficiaries([
                ...beneficiaries,
                { name: "", email: "", relationship: "", allocatedPercentage: 0 },
            ]);
        }
    };

    const removeBeneficiary = (index: number) => {
        if (beneficiaries.length > 1) {
            setBeneficiaries(beneficiaries.filter((_, i) => i !== index));
        }
    };

    const handleNext = () => {
        const stepKeys = STEPS.map((s) => s.key);
        const currentIndex = stepKeys.indexOf(step);
        if (currentIndex < stepKeys.length - 1) {
            setStep(stepKeys[currentIndex + 1]);
        }
    };

    const handleBack = () => {
        const stepKeys = STEPS.map((s) => s.key);
        const currentIndex = stepKeys.indexOf(step);
        if (currentIndex > 0) {
            setStep(stepKeys[currentIndex - 1]);
        }
    };

    return (
        <div
            className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
            onClick={onClose}
        >
            <div
                className="bg-[#161E22] border border-[#1C252A] rounded-2xl w-full max-w-2xl max-h-[90vh] overflow-hidden"
                onClick={(e) => e.stopPropagation()}
            >
                {/* Header */}
                <div className="flex items-center justify-between p-6 border-b border-[#1C252A]">
                    <h2 className="text-xl font-semibold text-[#FCFFFF]">
                        Create Future Plan
                    </h2>
                    <button
                        onClick={onClose}
                        className="text-[#92A5A8] hover:text-[#FCFFFF] transition-colors"
                    >
                        <X size={24} />
                    </button>
                </div>

                {/* Steps Indicator */}
                <div className="px-6 py-4 border-b border-[#1C252A]">
                    <div className="flex items-center justify-between">
                        {STEPS.map((s, i) => {
                            const isActive = i === currentStepIndex;
                            const isCompleted = i < currentStepIndex;

                            return (
                                <React.Fragment key={s.key}>
                                    <div className="flex flex-col items-center">
                                        <div
                                            className={`w-8 h-8 rounded-full flex items-center justify-center text-sm font-semibold transition-colors ${isCompleted
                                                    ? "bg-green-500 text-white"
                                                    : isActive
                                                        ? "bg-[#33C5E0] text-[#161E22]"
                                                        : "bg-[#1C252A] text-[#92A5A8] border border-[#2A3338]"
                                                }`}
                                        >
                                            {isCompleted ? <Check size={14} /> : i + 1}
                                        </div>
                                        <span
                                            className={`text-xs mt-1.5 font-medium ${isActive
                                                    ? "text-[#33C5E0]"
                                                    : isCompleted
                                                        ? "text-green-400"
                                                        : "text-[#92A5A8]"
                                                }`}
                                        >
                                            {s.label}
                                        </span>
                                    </div>
                                    {i < STEPS.length - 1 && (
                                        <div
                                            className={`flex-1 h-0.5 mx-2 mb-5 ${isCompleted ? "bg-green-500" : "bg-[#1C252A]"
                                                }`}
                                        />
                                    )}
                                </React.Fragment>
                            );
                        })}
                    </div>
                </div>

                {/* Body */}
                <div className="p-6 max-h-[50vh] overflow-y-auto">
                    {/* Step 1: Details */}
                    {step === "details" && (
                        <div className="space-y-4">
                            <div>
                                <label className="block text-sm text-[#92A5A8] mb-2">
                                    Plan Name *
                                </label>
                                <input
                                    type="text"
                                    value={planName}
                                    onChange={(e) => setPlanName(e.target.value)}
                                    className="w-full bg-[#1C252A] border border-[#2A3338] rounded-lg px-4 py-3 text-[#FCFFFF] placeholder-[#92A5A8] focus:outline-none focus:border-[#33C5E0]"
                                    placeholder="e.g., Wedding Fund, Tuition, or Inheritance"
                                />
                            </div>

                            <div>
                                <label className="block text-sm text-[#92A5A8] mb-2">
                                    Description *
                                </label>
                                <textarea
                                    value={planDescription}
                                    onChange={(e) => setPlanDescription(e.target.value)}
                                    className="w-full bg-[#1C252A] border border-[#2A3338] rounded-lg px-4 py-3 text-[#FCFFFF] placeholder-[#92A5A8] focus:outline-none focus:border-[#33C5E0] resize-none"
                                    placeholder="Describe your plan..."
                                    rows={3}
                                />
                            </div>

                            <div className="grid grid-cols-2 gap-4">
                                <div>
                                    <label className="block text-sm text-[#92A5A8] mb-2">
                                        Asset *
                                    </label>
                                    <select
                                        value={assetType}
                                        onChange={(e) => setAssetType(e.target.value)}
                                        className="w-full bg-[#1C252A] border border-[#2A3338] rounded-lg px-4 py-3 text-[#FCFFFF] focus:outline-none focus:border-[#33C5E0]"
                                    >
                                        <option value="ERC20_TOKEN1">ETH</option>
                                        <option value="ERC20_TOKEN2">BTC</option>
                                        <option value="ERC20_TOKEN3">USDT</option>
                                    </select>
                                </div>

                                <div>
                                    <label className="block text-sm text-[#92A5A8] mb-2">
                                        Amount *
                                    </label>
                                    <input
                                        type="number"
                                        value={assetAmount}
                                        onChange={(e) => setAssetAmount(e.target.value)}
                                        className="w-full bg-[#1C252A] border border-[#2A3338] rounded-lg px-4 py-3 text-[#FCFFFF] placeholder-[#92A5A8] focus:outline-none focus:border-[#33C5E0]"
                                        placeholder="0.00"
                                    />
                                </div>
                            </div>

                            <div className="grid grid-cols-2 gap-4">
                                <div>
                                    <label className="block text-sm text-[#92A5A8] mb-2">
                                        Distribution Method *
                                    </label>
                                    <select
                                        value={distributionMethod}
                                        onChange={(e) => setDistributionMethod(e.target.value)}
                                        className="w-full bg-[#1C252A] border border-[#2A3338] rounded-lg px-4 py-3 text-[#FCFFFF] focus:outline-none focus:border-[#33C5E0]"
                                    >
                                        <option value="LUMP_SUM">Lump Sum</option>
                                        <option value="MONTHLY">Monthly</option>
                                        <option value="QUARTERLY">Quarterly</option>
                                        <option value="YEARLY">Yearly</option>
                                    </select>
                                </div>

                                <div>
                                    <label className="block text-sm text-[#92A5A8] mb-2">
                                        Transfer Date *
                                    </label>
                                    <input
                                        type="date"
                                        value={transferDate}
                                        onChange={(e) => setTransferDate(e.target.value)}
                                        className="w-full bg-[#1C252A] border border-[#2A3338] rounded-lg px-4 py-3 text-[#FCFFFF] focus:outline-none focus:border-[#33C5E0]"
                                    />
                                </div>
                            </div>
                        </div>
                    )}

                    {/* Step 2: Beneficiaries */}
                    {step === "beneficiaries" && (
                        <div className="space-y-4">
                            <p className="text-sm text-[#92A5A8]">
                                Add up to 10 beneficiaries. Total allocation must equal 100%.
                            </p>

                            {beneficiaries.map((ben, index) => (
                                <div
                                    key={index}
                                    className="bg-[#1C252A] rounded-xl p-4 space-y-3"
                                >
                                    <div className="flex items-center justify-between">
                                        <span className="font-medium text-[#FCFFFF]">
                                            Beneficiary {index + 1}
                                        </span>
                                        {beneficiaries.length > 1 && (
                                            <button
                                                onClick={() => removeBeneficiary(index)}
                                                className="text-red-400 hover:text-red-300"
                                            >
                                                <Trash2 size={16} />
                                            </button>
                                        )}
                                    </div>

                                    <div className="grid grid-cols-2 gap-3">
                                        <input
                                            type="text"
                                            className="bg-[#161E22] border border-[#2A3338] rounded-lg px-3 py-2 text-[#FCFFFF] placeholder-[#92A5A8] focus:outline-none focus:border-[#33C5E0] text-sm"
                                            placeholder="Full Name"
                                        />
                                        <input
                                            type="email"
                                            className="bg-[#161E22] border border-[#2A3338] rounded-lg px-3 py-2 text-[#FCFFFF] placeholder-[#92A5A8] focus:outline-none focus:border-[#33C5E0] text-sm"
                                            placeholder="Email"
                                        />
                                        <input
                                            type="text"
                                            className="bg-[#161E22] border border-[#2A3338] rounded-lg px-3 py-2 text-[#FCFFFF] placeholder-[#92A5A8] focus:outline-none focus:border-[#33C5E0] text-sm"
                                            placeholder="Relationship"
                                        />
                                        <div className="relative">
                                            <input
                                                type="number"
                                                className="w-full bg-[#161E22] border border-[#2A3338] rounded-lg px-3 py-2 pr-8 text-[#FCFFFF] placeholder-[#92A5A8] focus:outline-none focus:border-[#33C5E0] text-sm"
                                                placeholder="Allocation"
                                            />
                                            <span className="absolute right-3 top-1/2 -translate-y-1/2 text-[#92A5A8]">
                                                %
                                            </span>
                                        </div>
                                    </div>
                                </div>
                            ))}

                            {beneficiaries.length < 10 && (
                                <button
                                    onClick={addBeneficiary}
                                    className="w-full flex items-center justify-center gap-2 bg-[#1C252A] text-[#33C5E0] py-3 rounded-lg hover:bg-[#2A3338] transition-colors"
                                >
                                    <Plus size={16} />
                                    Add Beneficiary
                                </button>
                            )}

                            <div className="flex items-center justify-between p-3 bg-[#1C252A] rounded-lg">
                                <span className="font-medium text-[#FCFFFF]">
                                    Total Allocation
                                </span>
                                <span className="font-bold text-[#33C5E0]">100%</span>
                            </div>
                        </div>
                    )}

                    {/* Step 3: Review */}
                    {step === "review" && (
                        <div className="space-y-4">
                            <div className="bg-[#1C252A] rounded-xl p-4 space-y-3">
                                <h3 className="font-semibold text-[#FCFFFF]">Plan Details</h3>
                                <div className="grid gap-2 text-sm">
                                    <div className="flex justify-between">
                                        <span className="text-[#92A5A8]">Name</span>
                                        <span className="text-[#FCFFFF]">
                                            {planName || "My Inheritance Plan"}
                                        </span>
                                    </div>
                                    <div className="flex justify-between">
                                        <span className="text-[#92A5A8]">Amount</span>
                                        <span className="text-[#FCFFFF]">
                                            {assetAmount || "1.0"} ETH
                                        </span>
                                    </div>
                                    <div className="flex justify-between">
                                        <span className="text-[#92A5A8]">Distribution</span>
                                        <span className="text-[#FCFFFF]">Lump Sum</span>
                                    </div>
                                    <div className="flex justify-between">
                                        <span className="text-[#92A5A8]">Transfer Date</span>
                                        <span className="text-[#FCFFFF]">
                                            {transferDate || "2025-12-31"}
                                        </span>
                                    </div>
                                    <div className="flex justify-between">
                                        <span className="text-[#92A5A8]">Fees (5%)</span>
                                        <span className="text-[#FCFFFF]">0.05 ETH</span>
                                    </div>
                                    <div className="flex justify-between font-medium pt-2 border-t border-[#2A3338]">
                                        <span className="text-[#FCFFFF]">Total Required</span>
                                        <span className="text-[#FCFFFF]">1.05 ETH</span>
                                    </div>
                                </div>
                            </div>

                            <div className="bg-[#1C252A] rounded-xl p-4 space-y-3">
                                <h3 className="font-semibold text-[#FCFFFF]">
                                    Beneficiaries ({beneficiaries.length})
                                </h3>
                                <div className="flex justify-between text-sm">
                                    <span className="text-[#FCFFFF]">John Doe (Son)</span>
                                    <span className="font-medium text-[#33C5E0]">60%</span>
                                </div>
                                <div className="flex justify-between text-sm">
                                    <span className="text-[#FCFFFF]">Jane Doe (Daughter)</span>
                                    <span className="font-medium text-[#33C5E0]">40%</span>
                                </div>
                            </div>

                            <div className="flex items-start gap-3 p-4 bg-blue-500/10 border border-blue-500/20 rounded-xl">
                                <span className="text-blue-400 text-sm">
                                    By creating this plan, you agree to lock your tokens in escrow
                                    until the transfer date.
                                </span>
                            </div>
                        </div>
                    )}

                    {/* Step 4: Approve */}
                    {step === "approve" && (
                        <div className="text-center py-8">
                            <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-[#33C5E0] flex items-center justify-center">
                                <Check className="text-[#161E22]" size={32} />
                            </div>
                            <h3 className="text-lg font-semibold text-[#FCFFFF]">
                                Ready to Approve Tokens
                            </h3>
                            <p className="text-[#92A5A8] mt-2 mb-4">
                                This will approve tokens and create your inheritance plan
                            </p>
                            <div className="bg-[#1C252A] p-3 rounded-lg mb-6 text-sm">
                                <div className="flex justify-between">
                                    <span className="text-[#92A5A8]">Amount to approve:</span>
                                    <span className="font-medium text-[#FCFFFF]">1.05 ETH</span>
                                </div>
                            </div>
                            <button
                                onClick={handleNext}
                                className="w-full bg-[#33C5E0] text-[#161E22] py-3 rounded-lg font-medium hover:bg-[#2AB8D3] transition-colors"
                            >
                                Start Transaction
                            </button>
                            <p className="text-xs text-[#92A5A8] mt-3">
                                You will need to confirm 2 transactions
                            </p>
                        </div>
                    )}

                    {/* Step 5: Create */}
                    {step === "create" && (
                        <div className="text-center py-8">
                            <Loader
                                className="animate-spin mx-auto text-[#33C5E0]"
                                size={48}
                            />
                            <h3 className="text-lg font-semibold text-[#FCFFFF] mt-4">
                                Creating Plan...
                            </h3>
                            <p className="text-[#92A5A8] mt-2">
                                Waiting for transaction confirmation...
                            </p>
                        </div>
                    )}
                </div>

                {/* Footer */}
                {["details", "beneficiaries", "review"].includes(step) && (
                    <div className="flex justify-between gap-4 p-6 border-t border-[#1C252A]">
                        {step !== "details" && (
                            <button
                                onClick={handleBack}
                                className="px-6 py-3 bg-[#1C252A] text-[#FCFFFF] rounded-lg hover:bg-[#2A3338] transition-colors"
                            >
                                Back
                            </button>
                        )}
                        <button
                            onClick={handleNext}
                            className="flex-1 bg-[#33C5E0] text-[#161E22] py-3 rounded-lg font-medium hover:bg-[#2AB8D3] transition-colors"
                        >
                            {step === "review" ? "Create Plan" : "Next"}
                        </button>
                    </div>
                )}
            </div>
        </div>
    );
}
