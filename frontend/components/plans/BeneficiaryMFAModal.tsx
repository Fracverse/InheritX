"use client";

import { useState, useEffect, useCallback } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  Fingerprint,
  KeyRound,
  Mail,
  X,
  Loader2,
  CheckCircle,
  AlertCircle,
  ShieldCheck,
  Smartphone,
} from "lucide-react";
import {
  isWebAuthnSupported,
  isPlatformAuthenticatorAvailable,
  registerWebAuthnCredential,
  verifyWebAuthnCredential,
  hasRegisteredPasskey,
  serializeVerificationResult,
} from "@/app/lib/webauthn";
import { plansAPI } from "@/app/lib/api/plans";
import type { Plan } from "@/app/lib/api/plans";

// ─── Types ───────────────────────────────────────────────────────────────────

type AuthMethod = "passkey" | "totp" | "select";
type AuthStep = "select" | "register" | "verify" | "totp" | "success" | "error";

interface BeneficiaryMFAModalProps {
  plan: Plan;
  open: boolean;
  onClose: () => void;
  onClaimSuccess: (plan: Plan) => void;
}

// ─── Component ──────────────────────────────────────────────────────────────

export function BeneficiaryMFAModal({
  plan,
  open,
  onClose,
  onClaimSuccess,
}: BeneficiaryMFAModalProps) {
  const [step, setStep] = useState<AuthStep>("select");
  const [authMethod, setAuthMethod] = useState<AuthMethod>("select");
  const [webauthnSupported, setWebauthnSupported] = useState(false);
  const [platformAvailable, setPlatformAvailable] = useState(false);
  const [hasPasskey, setHasPasskey] = useState(false);
  const [beneficiaryEmail, setBeneficiaryEmail] = useState("");
  const [beneficiaryName, setBeneficiaryName] = useState("");
  const [totpCode, setTotpCode] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  // ─── Check WebAuthn availability and existing passkey on mount ────────

  useEffect(() => {
    if (!open) return;
    const supported = isWebAuthnSupported();
    setWebauthnSupported(supported);
    if (supported) {
      isPlatformAuthenticatorAvailable().then(setPlatformAvailable);
      // Check if a passkey already exists using the WebAuthn Conditional UI API
      hasRegisteredPasskey().then(setHasPasskey);
    }
  }, [open]);

  // ─── Reset state when modal opens ────────────────────────────────────────

  useEffect(() => {
    if (open) {
      setStep("select");
      setAuthMethod("select");
      setError("");
      setTotpCode("");
    }
  }, [open, plan]);

  // ─── Handlers ─────────────────────────────────────────────────────────────

  const handleRegisterPasskey = useCallback(async () => {
    if (!beneficiaryEmail.trim()) {
      setError("Please enter your beneficiary email first.");
      return;
    }
    setLoading(true);
    setError("");
    try {
      const result = await registerWebAuthnCredential(
        beneficiaryEmail.trim(),
        beneficiaryName.trim() || plan.beneficiary_name || "Beneficiary"
      );
      setHasPasskey(true);
      setStep("verify");
    } catch (err) {
      setError(
        err instanceof Error
          ? err.message
          : "Failed to register passkey. Please try TOTP instead."
      );
      setStep("error");
    } finally {
      setLoading(false);
    }
  }, [beneficiaryEmail, beneficiaryName, plan]);

  const handleVerifyPasskey = useCallback(async () => {
    if (!beneficiaryEmail.trim()) {
      setError("Please enter your beneficiary email.");
      return;
    }
    setLoading(true);
    setError("");
    try {
      // Uses discoverable credentials — no credential ID stored in localStorage.
      // The browser resolves the correct passkey via the resident key.
      const result = await verifyWebAuthnCredential();
      const serialized = serializeVerificationResult(result);

      // Submit the claim with WebAuthn verification as the 2FA code
      // The backend receives the WebAuthn assertion as proof of identity
      const claimed = await plansAPI.claimPlan(plan.id, {
        beneficiary_email: beneficiaryEmail.trim(),
        two_fa_code: `webauthn:${serialized.credentialId}:${serialized.signature}`,
      });
      setStep("success");
      setTimeout(() => onClaimSuccess(claimed), 1500);
    } catch (err) {
      setError(
        err instanceof Error
          ? err.message
          : "Passkey verification failed. Please try TOTP fallback."
      );
      setStep("error");
    } finally {
      setLoading(false);
    }
  }, [beneficiaryEmail, plan, onClaimSuccess]);

  const handleTotpSubmit = useCallback(async () => {
    if (!beneficiaryEmail.trim()) {
      setError("Please enter your beneficiary email.");
      return;
    }
    if (totpCode.trim().length !== 6) {
      setError("Please enter the 6-digit code from your authenticator app.");
      return;
    }
    setLoading(true);
    setError("");
    try {
      const claimed = await plansAPI.claimPlan(plan.id, {
        beneficiary_email: beneficiaryEmail.trim(),
        two_fa_code: totpCode.trim(),
      });
      setStep("success");
      setTimeout(() => onClaimSuccess(claimed), 1500);
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Verification failed. Please try again."
      );
      setStep("error");
    } finally {
      setLoading(false);
    }
  }, [beneficiaryEmail, totpCode, plan, onClaimSuccess]);

  const handleMethodSelect = (method: AuthMethod) => {
    setAuthMethod(method);
    setError("");
    if (method === "passkey") {
      if (hasPasskey) {
        setStep("verify");
      } else {
        setStep("register");
      }
    } else if (method === "totp") {
      setStep("totp");
    }
  };

  // ─── Render ──────────────────────────────────────────────────────────────

  return (
    <AnimatePresence>
      {open && (
        <div
          className="fixed inset-0 z-50 flex items-end md:items-center justify-center"
          role="dialog"
          aria-modal="true"
          aria-label="Beneficiary Multi-Factor Authentication"
        >
          {/* Backdrop */}
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="absolute inset-0 bg-black/60 backdrop-blur-sm"
            onClick={onClose}
          />

          {/* Modal */}
          <motion.div
            initial={{ opacity: 0, y: 40 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 40 }}
            transition={{ type: "spring", damping: 26, stiffness: 300 }}
            className="relative w-full max-w-md max-h-[90dvh] overflow-y-auto mx-4 bg-[#161E22] border border-[#2A3338] rounded-2xl shadow-2xl"
          >
            {/* Header */}
            <div className="sticky top-0 z-10 flex items-center justify-between px-6 py-4 bg-[#161E22] border-b border-[#2A3338]">
              <div className="flex items-center gap-3">
                <div className="flex items-center justify-center w-10 h-10 rounded-xl bg-[#33C5E014] border border-[#33C5E040]">
                  <ShieldCheck size={20} className="text-[#33C5E0]" />
                </div>
                <div>
                  <h2 className="text-lg font-semibold text-white">
                    Beneficiary Verification
                  </h2>
                  <p className="text-xs text-[#92A5A8] mt-0.5">
                    Secure claim authentication
                  </p>
                </div>
              </div>
              <button
                onClick={onClose}
                aria-label="Close"
                className="p-2 rounded-lg text-[#92A5A8] hover:text-white hover:bg-[#1C252A] transition-colors"
              >
                <X size={18} />
              </button>
            </div>

            {/* Body */}
            <div className="px-6 py-5 space-y-5">
              {/* Plan info */}
              <div className="rounded-lg bg-[#0A0F11] border border-[#2A3338] p-3">
                <p className="text-xs text-[#92A5A8] uppercase tracking-wider">
                  Claiming Plan
                </p>
                <p className="text-sm text-slate-200 font-medium mt-1">
                  {plan.title}
                </p>
                {plan.beneficiary_name && (
                  <p className="text-xs text-[#92A5A8] mt-1">
                    Beneficiary: {plan.beneficiary_name}
                  </p>
                )}
              </div>

              {/* Beneficiary email input (shared across steps) */}
              {(step === "select" || step === "register" || step === "verify" || step === "totp") && (
                <div className="space-y-3">
                  <div className="flex flex-col gap-1">
                    <label className="text-xs text-[#92A5A8]">
                      Beneficiary Email
                    </label>
                    <div className="relative">
                      <Mail
                        size={14}
                        className="absolute left-3 top-1/2 -translate-y-1/2 text-[#4A5568]"
                      />
                      <input
                        type="email"
                        value={beneficiaryEmail}
                        onChange={(e) => setBeneficiaryEmail(e.target.value)}
                        placeholder="beneficiary@example.com"
                        className="w-full bg-[#0A0F11] border border-[#2A3338] rounded-lg pl-9 pr-4 py-2.5 text-sm text-slate-200 placeholder-[#4A5568] focus:outline-none focus:border-[#33C5E0] transition-colors"
                      />
                    </div>
                  </div>
                  {step === "register" && (
                    <div className="flex flex-col gap-1">
                      <label className="text-xs text-[#92A5A8]">
                        Beneficiary Name (optional)
                      </label>
                      <input
                        type="text"
                        value={beneficiaryName}
                        onChange={(e) => setBeneficiaryName(e.target.value)}
                        placeholder="Alice Smith"
                        className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-4 py-2.5 text-sm text-slate-200 placeholder-[#4A5568] focus:outline-none focus:border-[#33C5E0] transition-colors"
                      />
                    </div>
                  )}
                </div>
              )}

              {/* ─── Step: Select auth method ─── */}
              {step === "select" && (
                <div className="space-y-3">
                  <p className="text-xs text-[#92A5A8] text-center">
                    Choose your verification method
                  </p>

                  {/* Passkey option */}
                  <button
                    type="button"
                    onClick={() => handleMethodSelect("passkey")}
                    disabled={!webauthnSupported}
                    className="w-full flex items-center gap-3 p-4 rounded-xl border border-[#2A3338] bg-[#0A0F11] hover:border-[#33C5E0] hover:bg-[#33C5E008] transition-colors disabled:opacity-40 disabled:cursor-not-allowed text-left"
                  >
                    <div className="flex items-center justify-center w-10 h-10 rounded-lg bg-[#33C5E014]">
                      <Fingerprint size={20} className="text-[#33C5E0]" />
                    </div>
                    <div className="flex-1">
                      <p className="text-sm font-medium text-slate-200">
                        Passkey / WebAuthn
                      </p>
                      <p className="text-xs text-[#92A5A8] mt-0.5">
                        {webauthnSupported
                          ? hasPasskey
                            ? "Use your registered passkey"
                            : "Register a new passkey"
                          : "Not supported in this browser"}
                      </p>
                    </div>
                    {webauthnSupported && platformAvailable && (
                      <span className="text-[10px] text-[#48BB78] bg-[#48BB7814] px-2 py-1 rounded-full">
                        Recommended
                      </span>
                    )}
                  </button>

                  {/* TOTP fallback option */}
                  <button
                    type="button"
                    onClick={() => handleMethodSelect("totp")}
                    className="w-full flex items-center gap-3 p-4 rounded-xl border border-[#2A3338] bg-[#0A0F11] hover:border-[#33C5E0] hover:bg-[#33C5E008] transition-colors text-left"
                  >
                    <div className="flex items-center justify-center w-10 h-10 rounded-lg bg-[#F59E0B14]">
                      <KeyRound size={20} className="text-[#F59E0B]" />
                    </div>
                    <div className="flex-1">
                      <p className="text-sm font-medium text-slate-200">
                        Email 2FA / TOTP
                      </p>
                      <p className="text-xs text-[#92A5A8] mt-0.5">
                        Enter a 6-digit code from your email
                      </p>
                    </div>
                  </button>
                </div>
              )}

              {/* ─── Step: Register passkey ─── */}
              {step === "register" && (
                <div className="space-y-4">
                  <div className="flex items-center gap-3 p-3 rounded-lg bg-[#33C5E008] border border-[#33C5E030]">
                    <Smartphone size={16} className="text-[#33C5E0] flex-shrink-0" />
                    <p className="text-xs text-[#92A5A8]">
                      You'll be prompted by your browser to create a passkey
                      using your device's biometric or security key.
                    </p>
                  </div>
                  <button
                    type="button"
                    onClick={handleRegisterPasskey}
                    disabled={loading || !beneficiaryEmail.trim()}
                    className="w-full inline-flex items-center justify-center gap-2 px-5 py-3 text-sm font-medium text-black bg-[#33C5E0] hover:bg-cyan-300 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    {loading ? (
                      <>
                        <Loader2 size={16} className="animate-spin" />
                        Registering passkey…
                      </>
                    ) : (
                      <>
                        <Fingerprint size={16} />
                        Register Passkey
                      </>
                    )}
                  </button>
                  <button
                    type="button"
                    onClick={() => setStep("select")}
                    className="w-full text-xs text-[#92A5A8] hover:text-white transition-colors"
                  >
                    ← Back to methods
                  </button>
                </div>
              )}

              {/* ─── Step: Verify with passkey ─── */}
              {step === "verify" && (
                <div className="space-y-4">
                  <div className="flex flex-col items-center gap-3 py-4">
                    <div className="flex items-center justify-center w-16 h-16 rounded-full bg-[#33C5E014] border border-[#33C5E040]">
                      <Fingerprint size={32} className="text-[#33C5E0]" />
                    </div>
                    <p className="text-sm text-slate-200 text-center">
                      Verify your identity with your passkey
                    </p>
                    <p className="text-xs text-[#92A5A8] text-center">
                      Your browser will prompt for biometric or security key verification.
                    </p>
                  </div>
                  <button
                    type="button"
                    onClick={handleVerifyPasskey}
                    disabled={loading || !beneficiaryEmail.trim()}
                    className="w-full inline-flex items-center justify-center gap-2 px-5 py-3 text-sm font-medium text-black bg-[#33C5E0] hover:bg-cyan-300 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    {loading ? (
                      <>
                        <Loader2 size={16} className="animate-spin" />
                        Verifying…
                      </>
                    ) : (
                      <>
                        <ShieldCheck size={16} />
                        Verify & Claim
                      </>
                    )}
                  </button>
                  <button
                    type="button"
                    onClick={() => setStep("totp")}
                    className="w-full text-xs text-[#F59E0B] hover:text-amber-300 transition-colors"
                  >
                    Can't use passkey? Use TOTP instead →
                  </button>
                </div>
              )}

              {/* ─── Step: TOTP fallback ─── */}
              {step === "totp" && (
                <div className="space-y-4">
                  <div className="flex items-center gap-3 p-3 rounded-lg bg-[#F59E0B08] border border-[#F59E0B30]">
                    <Mail size={16} className="text-[#F59E0B] flex-shrink-0" />
                    <p className="text-xs text-[#92A5A8]">
                      A 6-digit verification code has been sent to your email.
                      Enter it below to complete the claim.
                    </p>
                  </div>
                  <div className="flex flex-col gap-1">
                    <label className="text-xs text-[#92A5A8]">
                      6-Digit Verification Code
                    </label>
                    <input
                      type="text"
                      inputMode="numeric"
                      pattern="[0-9]{6}"
                      maxLength={6}
                      value={totpCode}
                      onChange={(e) =>
                        setTotpCode(e.target.value.replace(/\D/g, ""))
                      }
                      placeholder="000000"
                      className="bg-[#0A0F11] border border-[#2A3338] rounded-lg px-4 py-3 text-sm text-slate-200 text-center font-mono text-lg tracking-[0.5em] placeholder-[#4A5568] focus:outline-none focus:border-[#33C5E0] transition-colors"
                    />
                  </div>
                  <button
                    type="button"
                    onClick={handleTotpSubmit}
                    disabled={loading || totpCode.length !== 6 || !beneficiaryEmail.trim()}
                    className="w-full inline-flex items-center justify-center gap-2 px-5 py-3 text-sm font-medium text-black bg-[#33C5E0] hover:bg-cyan-300 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    {loading ? (
                      <>
                        <Loader2 size={16} className="animate-spin" />
                        Verifying…
                      </>
                    ) : (
                      <>
                        <KeyRound size={16} />
                        Verify & Claim
                      </>
                    )}
                  </button>
                  {webauthnSupported && (
                    <button
                      type="button"
                      onClick={() => setStep(hasPasskey ? "verify" : "register")}
                      className="w-full text-xs text-[#33C5E0] hover:text-cyan-300 transition-colors"
                    >
                      ← Use passkey instead
                    </button>
                  )}
                </div>
              )}

              {/* ─── Step: Success ─── */}
              {step === "success" && (
                <div className="flex flex-col items-center gap-4 py-8">
                  <motion.div
                    initial={{ scale: 0 }}
                    animate={{ scale: 1 }}
                    transition={{ type: "spring", damping: 15 }}
                    className="flex items-center justify-center w-16 h-16 rounded-full bg-[#48BB7814] border border-[#48BB7840]"
                  >
                    <CheckCircle size={32} className="text-[#48BB78]" />
                  </motion.div>
                  <div className="text-center">
                    <p className="text-sm font-medium text-[#48BB78]">
                      Claim Verified Successfully
                    </p>
                    <p className="text-xs text-[#92A5A8] mt-1">
                      Your inheritance claim has been authenticated and submitted.
                    </p>
                  </div>
                </div>
              )}

              {/* ─── Error message ─── */}
              <AnimatePresence>
                {error && step !== "success" && (
                  <motion.div
                    initial={{ opacity: 0, y: -6 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0 }}
                    className="flex items-start gap-3 p-3 rounded-lg bg-[#F5656514] border border-[#F5656540] text-[#F56565] text-sm"
                  >
                    <AlertCircle size={16} className="mt-0.5 flex-shrink-0" />
                    <span>{error}</span>
                  </motion.div>
                )}
              </AnimatePresence>

              {/* ─── Error step actions ─── */}
              {step === "error" && (
                <div className="space-y-3">
                  <button
                    type="button"
                    onClick={() => setStep("select")}
                    className="w-full text-xs text-[#33C5E0] hover:text-cyan-300 transition-colors"
                  >
                    ← Try again
                  </button>
                </div>
              )}
            </div>
          </motion.div>
        </div>
      )}
    </AnimatePresence>
  );
}

export default BeneficiaryMFAModal;
