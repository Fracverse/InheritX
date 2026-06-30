"use client";
import { useWallet } from "@/context/WalletContext";
import { ConnectButton } from "@/components/ConnectButton";
import { LanguageSwitcher } from "@/components/LanguageSwitcher";
import { formatAddress } from "@/util/address";
import { useTranslations } from "next-intl";

export function DashboardHeader() {
  const { isConnected, address } = useWallet();
  const t = useTranslations("dashboardHeader");

  return (
    <header className="h-14 flex items-center justify-between px-6 border-b border-white/10 bg-[#0d1117] shrink-0">
      {/* Wallet status */}
      <div className="flex items-center gap-2">
        <span
          className={`w-2 h-2 rounded-full ${
            isConnected ? "bg-emerald-400 shadow-[0_0_6px_#34d399]" : "bg-gray-600"
          }`}
        />
        <span className="text-xs text-gray-500">
          {isConnected ? t("walletConnected") : t("walletDisconnected")}
        </span>
        {isConnected && address && (
          <span className="text-xs text-primary font-mono ml-1">
            {formatAddress(address)}
          </span>
        )}
      </div>

      {/* Right side */}
      <div className="flex items-center gap-3">
        <LanguageSwitcher />
        <ConnectButton targetUI="dashboard" />
      </div>
    </header>
  );
}
