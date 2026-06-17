"use client";

import React, { useCallback, useEffect, useMemo, useState } from "react";
import { Search, ChevronDown, ChevronUp, AlertCircle, CheckSquare, Square } from "lucide-react";
import { Skeleton } from "@/components/ui/Skeleton";

// ─── Types ───────────────────────────────────────────────────────────────────

export type SupportedChain = "ethereum" | "polygon" | "arbitrum" | "stellar" | "bitcoin";

export interface CrossChainAsset {
  chain: SupportedChain;
  contractAddress?: string;
  symbol: string;
  name: string;
  balance: string;
  decimals: number;
  usdValue?: number;
  logoUrl?: string;
}

export interface AssetsByChain {
  [chain: string]: CrossChainAsset[];
}

interface CrossChainAssetListProps {
  userAddress: string;
  onAssetSelect?: (asset: CrossChainAsset) => void;
  selectedAssets?: CrossChainAsset[];
  showSelection?: boolean;
}

// ─── Chain Config ─────────────────────────────────────────────────────────────

const CHAIN_CONFIG: Record<
  SupportedChain,
  { label: string; color: string; bgColor: string; borderColor: string; icon: string }
> = {
  ethereum: {
    label: "Ethereum",
    color: "text-blue-400",
    bgColor: "bg-blue-500/10",
    borderColor: "border-blue-500/20",
    icon: "Ξ",
  },
  polygon: {
    label: "Polygon",
    color: "text-purple-400",
    bgColor: "bg-purple-500/10",
    borderColor: "border-purple-500/20",
    icon: "⬡",
  },
  arbitrum: {
    label: "Arbitrum",
    color: "text-cyan-400",
    bgColor: "bg-cyan-500/10",
    borderColor: "border-cyan-500/20",
    icon: "◈",
  },
  stellar: {
    label: "Stellar",
    color: "text-gray-200",
    bgColor: "bg-gray-500/10",
    borderColor: "border-gray-500/20",
    icon: "✦",
  },
  bitcoin: {
    label: "Bitcoin",
    color: "text-orange-400",
    bgColor: "bg-orange-500/10",
    borderColor: "border-orange-500/20",
    icon: "₿",
  },
};

// ─── Sub-components ───────────────────────────────────────────────────────────

function AssetIcon({ asset }: { asset: CrossChainAsset }) {
  const chain = CHAIN_CONFIG[asset.chain];
  if (asset.logoUrl) {
    return (
      // eslint-disable-next-line @next/next/no-img-element
      <img
        src={asset.logoUrl}
        alt={asset.symbol}
        className="w-9 h-9 rounded-full object-cover"
        onError={(e) => {
          (e.currentTarget as HTMLImageElement).style.display = "none";
        }}
      />
    );
  }
  return (
    <div
      className={`w-9 h-9 rounded-full flex items-center justify-center text-sm font-bold ${chain.bgColor} ${chain.color}`}
      aria-label={asset.symbol}
    >
      {asset.symbol.slice(0, 3)}
    </div>
  );
}

function AssetCard({
  asset,
  isSelected,
  showSelection,
  onSelect,
}: {
  asset: CrossChainAsset;
  isSelected: boolean;
  showSelection: boolean;
  onSelect?: (asset: CrossChainAsset) => void;
}) {
  const formattedBalance = parseFloat(asset.balance).toLocaleString(undefined, {
    maximumFractionDigits: 6,
  });
  const formattedUsd =
    asset.usdValue != null
      ? asset.usdValue.toLocaleString("en-US", { style: "currency", currency: "USD" })
      : null;

  return (
    <div
      role={showSelection ? "checkbox" : undefined}
      aria-checked={showSelection ? isSelected : undefined}
      onClick={() => showSelection && onSelect?.(asset)}
      className={`flex items-center gap-3 p-3 rounded-xl border transition-colors ${
        showSelection ? "cursor-pointer" : ""
      } ${
        isSelected
          ? "bg-[#1C252A] border-[#2C8C7B]/60"
          : "bg-[#0A0F11] border-[#161E22] hover:border-[#2C3A3F]"
      }`}
    >
      <AssetIcon asset={asset} />
      <div className="flex-1 min-w-0">
        <p className="text-sm font-medium text-[#FCFFFF] truncate">{asset.symbol}</p>
        <p className="text-xs text-[#92A5A8] truncate">{asset.name}</p>
      </div>
      <div className="text-right shrink-0">
        <p className="text-sm font-medium text-[#FCFFFF]">{formattedBalance}</p>
        {formattedUsd && <p className="text-xs text-[#92A5A8]">{formattedUsd}</p>}
      </div>
      {showSelection && (
        <div className="ml-1 text-[#92A5A8]">
          {isSelected ? (
            <CheckSquare size={16} className="text-[#2C8C7B]" />
          ) : (
            <Square size={16} />
          )}
        </div>
      )}
    </div>
  );
}

function ChainGroup({
  chain,
  assets,
  selectedAssets,
  showSelection,
  onSelect,
}: {
  chain: SupportedChain;
  assets: CrossChainAsset[];
  selectedAssets: CrossChainAsset[];
  showSelection: boolean;
  onSelect?: (asset: CrossChainAsset) => void;
}) {
  const [collapsed, setCollapsed] = useState(false);
  const config = CHAIN_CONFIG[chain];
  const chainUsd = assets.reduce((sum, a) => sum + (a.usdValue ?? 0), 0);
  const formattedChainUsd = chainUsd > 0
    ? chainUsd.toLocaleString("en-US", { style: "currency", currency: "USD" })
    : null;

  const isSelected = (asset: CrossChainAsset) =>
    selectedAssets.some(
      (s) => s.chain === asset.chain && s.symbol === asset.symbol && s.contractAddress === asset.contractAddress
    );

  return (
    <div className={`border rounded-2xl overflow-hidden ${config.borderColor}`}>
      <button
        onClick={() => setCollapsed((c) => !c)}
        className={`w-full flex items-center gap-3 p-4 ${config.bgColor} hover:brightness-110 transition-all`}
        aria-expanded={!collapsed}
      >
        <span className={`text-xl font-bold ${config.color}`}>{config.icon}</span>
        <span className={`font-semibold text-sm ${config.color}`}>{config.label}</span>
        <span className="ml-1 text-xs text-[#92A5A8]">({assets.length})</span>
        {formattedChainUsd && (
          <span className="ml-auto text-sm text-[#92A5A8] mr-2">{formattedChainUsd}</span>
        )}
        {collapsed ? (
          <ChevronDown size={14} className="text-[#92A5A8]" />
        ) : (
          <ChevronUp size={14} className="text-[#92A5A8]" />
        )}
      </button>
      {!collapsed && (
        <div className="p-3 space-y-2 bg-[#0A0F11]">
          {assets.map((asset) => (
            <AssetCard
              key={`${asset.chain}-${asset.symbol}-${asset.contractAddress ?? ""}`}
              asset={asset}
              isSelected={isSelected(asset)}
              showSelection={showSelection}
              onSelect={onSelect}
            />
          ))}
        </div>
      )}
    </div>
  );
}

// ─── Main Component ───────────────────────────────────────────────────────────

export const CrossChainAssetList: React.FC<CrossChainAssetListProps> = ({
  userAddress,
  onAssetSelect,
  selectedAssets = [],
  showSelection = false,
}) => {
  const [assetsByChain, setAssetsByChain] = useState<AssetsByChain>({});
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searchFilter, setSearchFilter] = useState("");
  const [chainFilter, setChainFilter] = useState<SupportedChain[]>([]);
  const [sortBy, setSortBy] = useState<"value" | "balance" | "alpha">("value");
  const [retryCount, setRetryCount] = useState(0);

  const fetchAssets = useCallback(async () => {
    if (!userAddress) return;
    setLoading(true);
    setError(null);
    try {
      const res = await fetch(`/api/cross-chain/assets/${userAddress}`);
      if (!res.ok) throw new Error(`Failed to fetch assets (${res.status})`);
      const data: AssetsByChain = await res.json();
      setAssetsByChain(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load assets");
    } finally {
      setLoading(false);
    }
  }, [userAddress]);

  useEffect(() => {
    fetchAssets();
  }, [fetchAssets, retryCount]);

  const filteredAndSorted = useMemo<AssetsByChain>(() => {
    const result: AssetsByChain = {};
    const search = searchFilter.toLowerCase();

    for (const [chain, assets] of Object.entries(assetsByChain)) {
      if (chainFilter.length > 0 && !chainFilter.includes(chain as SupportedChain)) continue;

      const filtered = assets.filter(
        (a) =>
          a.symbol.toLowerCase().includes(search) || a.name.toLowerCase().includes(search)
      );

      if (filtered.length === 0) continue;

      const sorted = [...filtered].sort((a, b) => {
        if (sortBy === "value") return (b.usdValue ?? 0) - (a.usdValue ?? 0);
        if (sortBy === "balance") return parseFloat(b.balance) - parseFloat(a.balance);
        return a.symbol.localeCompare(b.symbol);
      });

      result[chain] = sorted;
    }
    return result;
  }, [assetsByChain, searchFilter, chainFilter, sortBy]);

  const availableChains = useMemo(
    () => Object.keys(assetsByChain) as SupportedChain[],
    [assetsByChain]
  );

  const totalAssets = useMemo(
    () => Object.values(filteredAndSorted).reduce((s, a) => s + a.length, 0),
    [filteredAndSorted]
  );

  const toggleChainFilter = (chain: SupportedChain) => {
    setChainFilter((prev) =>
      prev.includes(chain) ? prev.filter((c) => c !== chain) : [...prev, chain]
    );
  };

  if (loading) {
    return (
      <div className="space-y-4" aria-label="Loading assets" aria-busy="true">
        {[1, 2, 3].map((i) => (
          <div key={i} className="border border-[#161E22] rounded-2xl overflow-hidden">
            <div className="p-4 bg-[#1C252A] flex items-center gap-3">
              <Skeleton className="w-7 h-7 rounded-full" />
              <Skeleton className="h-4 w-24" />
            </div>
            <div className="p-3 space-y-2 bg-[#0A0F11]">
              {[1, 2].map((j) => (
                <div key={j} className="flex items-center gap-3 p-3">
                  <Skeleton className="w-9 h-9 rounded-full" />
                  <div className="flex-1 space-y-1">
                    <Skeleton className="h-4 w-16" />
                    <Skeleton className="h-3 w-24" />
                  </div>
                  <div className="text-right space-y-1">
                    <Skeleton className="h-4 w-20" />
                    <Skeleton className="h-3 w-16" />
                  </div>
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
    );
  }

  if (error) {
    return (
      <div
        role="alert"
        className="flex flex-col items-center gap-3 py-10 text-center"
      >
        <AlertCircle size={32} className="text-red-400" />
        <p className="text-[#FCFFFF] font-medium">Failed to load assets</p>
        <p className="text-sm text-[#92A5A8]">{error}</p>
        <button
          onClick={() => setRetryCount((c) => c + 1)}
          className="mt-2 px-4 py-2 text-sm rounded-full bg-[#1C252A] text-[#FCFFFF] border border-[#2C3A3F] hover:border-[#2C8C7B] transition-colors"
        >
          Retry
        </button>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {/* Search + Sort controls */}
      <div className="flex flex-col sm:flex-row gap-2">
        <div className="relative flex-1">
          <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-[#92A5A8]" />
          <input
            type="text"
            placeholder="Search assets…"
            value={searchFilter}
            onChange={(e) => setSearchFilter(e.target.value)}
            className="w-full pl-8 pr-3 py-2 text-sm bg-[#1C252A] border border-[#2C3A3F] rounded-lg text-[#FCFFFF] placeholder-[#92A5A8] focus:outline-none focus:border-[#2C8C7B]"
          />
        </div>
        <select
          value={sortBy}
          onChange={(e) => setSortBy(e.target.value as typeof sortBy)}
          aria-label="Sort assets"
          className="px-3 py-2 text-sm bg-[#1C252A] border border-[#2C3A3F] rounded-lg text-[#FCFFFF] focus:outline-none focus:border-[#2C8C7B]"
        >
          <option value="value">Sort: Value</option>
          <option value="balance">Sort: Balance</option>
          <option value="alpha">Sort: A–Z</option>
        </select>
      </div>

      {/* Chain filter pills */}
      {availableChains.length > 1 && (
        <div className="flex flex-wrap gap-2">
          {availableChains.map((chain) => {
            const config = CHAIN_CONFIG[chain];
            const active = chainFilter.includes(chain);
            return (
              <button
                key={chain}
                onClick={() => toggleChainFilter(chain)}
                aria-pressed={active}
                className={`px-3 py-1 rounded-full text-xs font-medium border transition-colors ${
                  active
                    ? `${config.bgColor} ${config.color} ${config.borderColor}`
                    : "bg-[#1C252A] text-[#92A5A8] border-[#2C3A3F] hover:border-[#2C8C7B]"
                }`}
              >
                {config.label}
              </button>
            );
          })}
        </div>
      )}

      {/* Asset groups */}
      {totalAssets === 0 ? (
        <div className="flex flex-col items-center gap-2 py-10 text-center" role="status">
          <p className="text-[#FCFFFF] font-medium">No assets found</p>
          <p className="text-sm text-[#92A5A8]">
            {searchFilter || chainFilter.length > 0
              ? "Try adjusting your search or filters."
              : "No assets detected for this address."}
          </p>
        </div>
      ) : (
        <div className="space-y-3">
          {(Object.entries(filteredAndSorted) as [SupportedChain, CrossChainAsset[]][]).map(
            ([chain, assets]) => (
              <ChainGroup
                key={chain}
                chain={chain}
                assets={assets}
                selectedAssets={selectedAssets}
                showSelection={showSelection}
                onSelect={onAssetSelect}
              />
            )
          )}
        </div>
      )}
    </div>
  );
};
