"use client";
import { useWallet } from "@/context/WalletContext";
import { formatAddress } from "@/util/address";

export default function AssetOwnerPage() {
  const { isConnected, address, openModal } = useWallet();

  return (
    <div className="animate-fade-in">
      {/* Page heading */}
      <div className="mb-8">
        <h1 className="text-2xl font-semibold text-foreground">Overview</h1>
        <p className="text-sm text-gray-500 mt-1">
          Welcome to your InheritX dashboard.
        </p>
      </div>

      {/* Wallet status card */}
      <div className="bg-white/3 border border-white/10 rounded-2xl p-6 max-w-md">
        <p className="text-xs text-gray-500 uppercase tracking-wider mb-4">
          Wallet Status
        </p>

        {isConnected && address ? (
          <div className="flex flex-col gap-3">
            <div className="flex items-center gap-2">
              <span className="w-2 h-2 rounded-full bg-emerald-400 shadow-[0_0_6px_#34d399]" />
              <span className="text-sm text-emerald-400 font-medium">
                Connected
              </span>
            </div>
            <div className="bg-white/5 border border-white/10 rounded-lg px-4 py-3">
              <p className="text-xs text-gray-500 mb-1">Wallet Address</p>
              <p className="text-sm font-mono text-primary">
                {formatAddress(address)}
              </p>
            </div>
          </div>
        ) : (
          <div className="flex flex-col gap-4">
            <div className="flex items-center gap-2">
              <span className="w-2 h-2 rounded-full bg-gray-600" />
              <span className="text-sm text-gray-500">Not connected</span>
            </div>
            <button
              onClick={openModal}
              className="px-4 py-2.5 text-sm font-medium rounded-lg bg-primary text-black hover:bg-primary/90 transition-colors w-fit"
            >
              Connect Wallet
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
