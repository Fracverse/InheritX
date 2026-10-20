"use client";

import React, { useState, useCallback, useEffect } from "react";
import {
  KeyRound,
  Eye,
  EyeOff,
  Copy,
  Check,
  AlertTriangle,
  FlaskConical,
  Shuffle,
  ArrowRight,
} from "lucide-react";

const IS_DEV =
  process.env.NODE_ENV === "development" ||
  process.env.NEXT_PUBLIC_APP_ENV === "development" ||
  process.env.NEXT_PUBLIC_ENABLE_DEV_WALLETS === "true";

const TESTNET_ACCOUNTS = [
  {
    label: "QA Account 1 (Funded)",
    secret: "SCZANGBA5YHTNYVVV7C338RBV4HU5LDEHQI4BID3O6SN7BXHC5FE63Q5",
    public: "GBRPYHIL2CI3FNQ4BXLFMNDLFJUNPU2HY3ZMFSHONUCEOASW7QC7OX2H",
  },
  {
    label: "QA Account 2 (Funded)",
    secret: "SAKS75OVSULZQQKCNPCUWIKJTGZBF5B2I34PBAVHYI7J3V4ZD6VBHSLQ",
    public: "GAOO3LWBC4XF6VWRP5ESJ6IBHAISVJMSBTALHOQM2EZG7Q477UWA6L7U",
  },
  {
    label: "QA Account 3 (Fresh)",
    secret: "SCAVV6YQANUFYOFQCOEAAZSKUMK4K5JZ3N5V6DQ5J4H2V3L4A2ZB6H3R",
    public: "GAVBC7S4J4Y73NFR5XKFPO3KJ4J5M6QJ3H7A2ZB6C8D9E0F1G2H3J4K5L",
  },
];

interface DevKeypairConnectProps {
  onConnect: (secretKey: string, publicKey: string) => void;
}

export function DevKeypairConnect({ onConnect }: DevKeypairConnectProps) {
  const [selectedAccount, setSelectedAccount] = useState(0);
  const [showSecret, setShowSecret] = useState(false);
  const [customSecret, setCustomSecret] = useState("");
  const [useCustom, setUseCustom] = useState(false);
  const [copied, setCopied] = useState(false);
  const [isDev, setIsDev] = useState(false);

  useEffect(() => {
    setIsDev(IS_DEV);
  }, []);

  if (!isDev) return null;

  const account = TESTNET_ACCOUNTS[selectedAccount];
  const secretToUse = useCustom ? customSecret : account.secret;

  const handleConnect = () => {
    if (!secretToUse || secretToUse.length < 56) return;
    onConnect(secretToUse, useCustom ? "" : account.public);
  };

  const handleCopy = () => {
    navigator.clipboard.writeText(secretToUse).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  };

  const handleShuffle = () => {
    setSelectedAccount((prev) => (prev + 1) % TESTNET_ACCOUNTS.length);
    setUseCustom(false);
  };

  return (
    <div
      className="mt-4 pt-4 border-t border-white/[0.06]"
      data-testid="dev-keypair-connect"
    >
      {/* Dev badge */}
      <div className="flex items-center gap-2 mb-3">
        <FlaskConical size={12} className="text-yellow-400" />
        <span className="text-[10px] font-medium text-yellow-400 uppercase tracking-wider">
          Development Mode
        </span>
        <span className="text-[10px] text-slate-600">· E2E Testing</span>
      </div>

      {/* Mode toggle */}
      <div className="flex gap-1 p-1 rounded-lg bg-white/[0.03] border border-white/[0.05] mb-3">
        <button
          onClick={() => setUseCustom(false)}
          className={`flex-1 px-2 py-1.5 rounded-md text-[11px] font-medium transition-colors ${
            !useCustom ? "bg-[#33C5E0]/10 text-[#33C5E0]" : "text-slate-500 hover:text-slate-300"
          }`}
        >
          Testnet Accounts
        </button>
        <button
          onClick={() => setUseCustom(true)}
          className={`flex-1 px-2 py-1.5 rounded-md text-[11px] font-medium transition-colors ${
            useCustom ? "bg-[#33C5E0]/10 text-[#33C5E0]" : "text-slate-500 hover:text-slate-300"
          }`}
        >
          Custom Key
        </button>
      </div>

      {!useCustom ? (
        <>
          {/* Testnet account selector */}
          <div className="space-y-1.5 mb-3">
            {TESTNET_ACCOUNTS.map((acc, i) => (
              <button
                key={i}
                onClick={() => setSelectedAccount(i)}
                className={`w-full flex items-center justify-between px-3 py-2 rounded-lg text-[11px] transition-colors ${
                  i === selectedAccount
                    ? "bg-[#33C5E0]/10 border border-[#33C5E0]/20 text-white"
                    : "bg-white/[0.02] border border-transparent text-slate-400 hover:bg-white/[0.04]"
                }`}
              >
                <span>{acc.label}</span>
                <span className="font-mono text-[9px] text-slate-500">
                  {acc.public.slice(0, 8)}…
                </span>
              </button>
            ))}
          </div>

          {/* Secret key display */}
          <div className="relative mb-3">
            <input
              type={showSecret ? "text" : "password"}
              value={account.secret}
              readOnly
              className="w-full px-3 py-2 pr-20 rounded-lg bg-white/[0.03] border border-white/[0.05] text-[10px] font-mono text-slate-300"
              aria-label="Testnet secret key"
            />
            <div className="absolute right-1 top-1/2 -translate-y-1/2 flex gap-0.5">
              <button
                onClick={() => setShowSecret(!showSecret)}
                className="p-1.5 text-slate-500 hover:text-slate-300 transition-colors"
                aria-label={showSecret ? "Hide secret" : "Show secret"}
              >
                {showSecret ? <EyeOff size={12} /> : <Eye size={12} />}
              </button>
              <button
                onClick={handleCopy}
                className="p-1.5 text-slate-500 hover:text-slate-300 transition-colors"
                aria-label="Copy secret key"
              >
                {copied ? <Check size={12} className="text-emerald-400" /> : <Copy size={12} />}
              </button>
            </div>
          </div>

          <button
            onClick={handleShuffle}
            className="flex items-center gap-1.5 text-[11px] text-slate-500 hover:text-slate-300 transition-colors mb-3"
          >
            <Shuffle size={10} />
            Cycle to next account
          </button>
        </>
      ) : (
        <>
          {/* Custom secret input */}
          <div className="relative mb-3">
            <input
              type={showSecret ? "text" : "password"}
              value={customSecret}
              onChange={(e) => setCustomSecret(e.target.value)}
              placeholder="SC... (56-char Stellar secret key)"
              className="w-full px-3 py-2 pr-10 rounded-lg bg-white/[0.03] border border-white/[0.08] text-[10px] font-mono text-slate-300 placeholder-slate-600 focus:outline-none focus:border-[#33C5E0]/40"
              aria-label="Custom secret key"
              maxLength={56}
            />
            <button
              onClick={() => setShowSecret(!showSecret)}
              className="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-slate-500 hover:text-slate-300"
              aria-label={showSecret ? "Hide secret" : "Show secret"}
            >
              {showSecret ? <EyeOff size={12} /> : <Eye size={12} />}
            </button>
          </div>
          {customSecret && customSecret.length < 56 && (
            <p className="text-[10px] text-yellow-400 flex items-center gap-1 mb-2">
              <AlertTriangle size={9} />
              Secret key must be 56 characters
            </p>
          )}
        </>
      )}

      {/* Connect button */}
      <button
        onClick={handleConnect}
        disabled={!secretToUse || secretToUse.length < 56}
        className="w-full py-2.5 rounded-lg bg-[#33C5E0]/10 text-[#33C5E0] border border-[#33C5E0]/20 text-xs font-semibold hover:bg-[#33C5E0]/20 transition-colors disabled:opacity-40 disabled:cursor-not-allowed flex items-center justify-center gap-2"
        data-testid="dev-connect-btn"
      >
        <KeyRound size={13} />
        Quick Connect (Dev)
        <ArrowRight size={13} />
      </button>

      <p className="text-[9px] text-slate-600 text-center mt-2">
        Testnet only. Never use production keys here.
      </p>
    </div>
  );
}
