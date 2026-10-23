"use client";

import { useKYC } from "@/context/KYCContext";
import { KYCVerificationModal } from "@/components/kyc/KYCVerificationModal";
import { useState } from "react";
import { useWallet } from "@/context/WalletContext";
import {
  ShieldCheck,
  ShieldAlert,
  Clock,
  CheckCircle2,
  XCircle,
  AlertCircle,
  ChevronRight,
  User,
  FileText,
  Wallet,
  ArrowRight,
} from "lucide-react";

function KYCStatusCard({ status }: { status: string }) {
  const config: Record<string, { icon: React.ElementType; label: string; desc: string; cls: string; iconCls: string }> = {
    approved: {
      icon: CheckCircle2,
      label: "KYC Approved",
      desc: "Your identity has been verified. You have full access to all InheritX features.",
      cls: "border-emerald-500/20 bg-emerald-500/5",
      iconCls: "text-emerald-400",
    },
    pending: {
      icon: Clock,
      label: "Verification Pending",
      desc: "Your documents are being reviewed. This typically takes 1–2 business days.",
      cls: "border-yellow-500/20 bg-yellow-500/5",
      iconCls: "text-yellow-400",
    },
    rejected: {
      icon: XCircle,
      label: "Verification Rejected",
      desc: "Your KYC submission was rejected. Please review the reason and resubmit.",
      cls: "border-red-500/20 bg-red-500/5",
      iconCls: "text-red-400",
    },
    not_started: {
      icon: ShieldAlert,
      label: "Not Verified",
      desc: "Complete identity verification to unlock creating and claiming inheritance plans.",
      cls: "border-slate-500/20 bg-slate-500/5",
      iconCls: "text-slate-400",
    },
  };

  const c = config[status] ?? config.not_started;
  const Icon = c.icon;

  return (
    <div className={`rounded-2xl border p-5 flex items-start gap-4 ${c.cls}`}>
      <div className="w-10 h-10 rounded-xl bg-white/5 flex items-center justify-center flex-shrink-0">
        <Icon size={20} className={c.iconCls} />
      </div>
      <div>
        <p className="text-sm font-semibold text-white">{c.label}</p>
        <p className="text-xs text-slate-400 mt-1 leading-relaxed">{c.desc}</p>
      </div>
    </div>
  );
}

const STEPS = [
  { icon: User, label: "Personal Info", desc: "Name, date of birth, nationality" },
  { icon: FileText, label: "Identity Document", desc: "Passport, driver's licence, or ID card" },
  { icon: FileText, label: "Address Proof", desc: "Utility bill or bank statement" },
  { icon: CheckCircle2, label: "Review & Submit", desc: "Final check before submission" },
];

export default function KYCPage() {
  const { isConnected, openModal } = useWallet();
  const { kycStatus, isKYCModalOpen, openKYCModal } = useKYC();

  return (
    <div className="animate-fade-in space-y-5 max-w-2xl">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-semibold text-white">KYC Verification</h1>
        <p className="text-sm text-slate-500 mt-1">
          Identity verification is required to create or claim inheritance plans on InheritX.
        </p>
      </div>

      {/* Status Card */}
      <KYCStatusCard status={kycStatus ?? "not_started"} />

      {/* Wallet Guard */}
      {!isConnected && (
        <div className="rounded-2xl border border-white/10 bg-white/[0.03] p-6 flex items-center gap-4">
          <Wallet size={20} className="text-[#33C5E0] flex-shrink-0" />
          <div className="flex-1">
            <p className="text-sm font-medium text-white">Connect your wallet first</p>
            <p className="text-xs text-slate-400 mt-0.5">A wallet connection is required to submit KYC.</p>
          </div>
          <button
            onClick={openModal}
            className="px-4 py-2 rounded-xl bg-[#33C5E0] text-black text-sm font-semibold flex-shrink-0"
          >
            Connect
          </button>
        </div>
      )}

      {/* Process Overview */}
      <div className="rounded-2xl bg-white/[0.03] border border-white/[0.08] p-5">
        <h2 className="text-sm font-semibold text-white mb-4">Verification Process</h2>
        <div className="space-y-3">
          {STEPS.map((step, i) => {
            const Icon = step.icon;
            const isDone = kycStatus === "approved";
            return (
              <div key={i} className="flex items-center gap-3">
                <div className={`w-7 h-7 rounded-full flex items-center justify-center flex-shrink-0 border ${
                  isDone
                    ? "bg-emerald-500/10 border-emerald-500/20"
                    : "bg-white/5 border-white/10"
                }`}>
                  {isDone
                    ? <CheckCircle2 size={13} className="text-emerald-400" />
                    : <span className="text-[10px] text-slate-500 font-semibold">{i + 1}</span>
                  }
                </div>
                <div className="flex-1">
                  <p className={`text-sm font-medium ${isDone ? "text-emerald-400" : "text-slate-300"}`}>{step.label}</p>
                  <p className="text-xs text-slate-500">{step.desc}</p>
                </div>
                {i < STEPS.length - 1 && (
                  <div className="absolute left-[27px] mt-7 w-px h-3 bg-white/10" />
                )}
              </div>
            );
          })}
        </div>
      </div>

      {/* Action Button */}
      {isConnected && kycStatus !== "approved" && (
        <button
          onClick={openKYCModal}
          className="w-full flex items-center justify-center gap-2 py-3.5 rounded-xl bg-[#33C5E0] text-black font-semibold text-sm hover:bg-[#33C5E0]/90 transition-colors active:scale-[0.98]"
        >
          <ShieldCheck size={16} />
          {kycStatus === "pending" ? "Check Verification Status" : kycStatus === "rejected" ? "Resubmit Documents" : "Start Verification"}
          <ArrowRight size={14} />
        </button>
      )}

      {kycStatus === "approved" && (
        <div className="rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-4 text-center">
          <CheckCircle2 size={24} className="text-emerald-400 mx-auto mb-2" />
          <p className="text-sm font-medium text-emerald-300">You&apos;re fully verified</p>
          <p className="text-xs text-slate-400 mt-1">All features are unlocked for your account.</p>
        </div>
      )}

      {/* Why KYC? */}
      <div className="rounded-2xl bg-white/[0.03] border border-white/[0.08] p-5">
        <h2 className="text-sm font-semibold text-white mb-3">Why is KYC required?</h2>
        <div className="space-y-3 text-xs text-slate-400 leading-relaxed">
          <p>• <span className="text-slate-300">Regulatory compliance:</span> InheritX operates under financial regulations that require identity verification for inheritance and asset transfers.</p>
          <p>• <span className="text-slate-300">Fiat off-ramp:</span> Beneficiaries receiving funds via Stellar anchors into bank accounts or mobile money must be verified individuals.</p>
          <p>• <span className="text-slate-300">Fraud prevention:</span> KYC ensures only legitimate heirs can claim plans, protecting your legacy.</p>
        </div>
      </div>

      {/* KYC Modal */}
      {isKYCModalOpen && <KYCVerificationModal />}
    </div>
  );
}
