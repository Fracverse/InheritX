'use client';

import { useState } from 'react';
import { Info, TrendingUp } from 'lucide-react';
import { useYieldCalculations } from '@/app/hooks/useYieldCalculations';
import { YieldChart } from './YieldChart';

interface TokenRateConfig {
  name: string;
  displayName: string;
  rateBps: number;
  description?: string;
}

interface YieldCalculatorWidgetProps {
  /** Initial principal amount */
  initialAmount?: number;
  /** Initial holding period in years */
  initialYears?: number;
  /** Token yield rate configurations */
  tokenRates?: TokenRateConfig[];
  /** Currency symbol for display */
  currency?: string;
  /** Callback when values change */
  onChange?: (amount: number, years: number) => void;
  /** Show comparison of multiple tokens */
  showComparison?: boolean;
  /** Whether to show the widget in compact mode */
  compact?: boolean;
}

// Default token configurations
const DEFAULT_TOKEN_RATES: TokenRateConfig[] = [
  { name: 'XLM', displayName: 'Stellar (XLM)', rateBps: 200, description: '2% annual yield' },
  { name: 'USDC', displayName: 'USDC', rateBps: 300, description: '3% annual yield' },
  { name: 'CUSTOM', displayName: 'Custom Token', rateBps: 100, description: '1% annual yield' },
];

/**
 * Tooltip component for information icons
 */
function InfoTooltip({ text }: { text: string }) {
  const [isVisible, setIsVisible] = useState(false);

  return (
    <div className="relative inline-block">
      <button
        type="button"
        onMouseEnter={() => setIsVisible(true)}
        onMouseLeave={() => setIsVisible(false)}
        onClick={() => setIsVisible(!isVisible)}
        className="inline-flex items-center justify-center w-4 h-4 ml-1 rounded-full bg-white/10 hover:bg-white/20 transition-colors"
      >
        <Info size={12} className="text-gray-400" />
      </button>

      {isVisible && (
        <div className="absolute z-10 bottom-full left-1/2 transform -translate-x-1/2 mb-2 px-3 py-2 text-xs text-slate-100 bg-slate-900 rounded-lg border border-white/20 whitespace-nowrap pointer-events-none">
          {text}
          <div className="absolute top-full left-1/2 transform -translate-x-1/2 border-4 border-transparent border-t-slate-900" />
        </div>
      )}
    </div>
  );
}

/**
 * Yield Calculator Widget - Shows projected yield over time with interactive charts and token comparison
 */
export function YieldCalculatorWidget({
  initialAmount = 10000,
  initialYears = 5,
  tokenRates = DEFAULT_TOKEN_RATES,
  currency = '$',
  onChange,
  showComparison = true,
  compact = false,
}: YieldCalculatorWidgetProps) {
  const [principal, setPrincipal] = useState(initialAmount);
  const [years, setYears] = useState(initialYears);
  const [selectedTokenIndex, setSelectedTokenIndex] = useState(0);

  // Calculate yield for the selected token
  const selectedCalculation = useYieldCalculations(
    principal,
    years,
    tokenRates[selectedTokenIndex]?.rateBps || 0
  );

  const selectedToken = tokenRates[selectedTokenIndex];

  // Notify parent component of changes
  const handleAmountChange = (value: string) => {
    const amount = parseFloat(value) || 0;
    setPrincipal(amount);
    onChange?.(amount, years);
  };

  const handleYearsChange = (value: string) => {
    const y = Math.min(Math.max(parseInt(value) || 1, 1), 50);
    setYears(y);
    onChange?.(principal, y);
  };

  if (compact) {
    // Compact mode - just show summary
    return (
      <div className="space-y-3 rounded-lg border border-white/10 bg-white/[0.03] p-4">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm font-medium text-slate-200">Projected Yield</p>
            <p className="mt-1 text-xs text-gray-500">
              {years} years at {(selectedToken.rateBps / 100).toFixed(2)}% APY
            </p>
          </div>
          <div className="text-right">
            <p className="text-lg font-bold text-emerald-300">
              +{currency}
              {selectedCalculation.finalAccruedYield.toLocaleString('en-US', {
                minimumFractionDigits: 2,
                maximumFractionDigits: 2,
              })}
            </p>
            <p className="text-xs text-gray-400 mt-1">
              Total: {currency}
              {selectedCalculation.finalTotal.toLocaleString('en-US', {
                minimumFractionDigits: 2,
                maximumFractionDigits: 2,
              })}
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6 rounded-lg border border-white/10 bg-white/[0.03] p-6">
      {/* Header */}
      <div>
        <div className="flex items-center gap-2">
          <TrendingUp size={20} className="text-primary" />
          <h3 className="text-lg font-semibold text-slate-100">Yield Accrual Calculator</h3>
        </div>
        <p className="mt-1 text-sm text-gray-500">
          See how your assets grow over time with compound interest using our yield-optimized settings.
        </p>
      </div>

      {/* Input Controls */}
      <div className="grid gap-4 sm:grid-cols-2">
        {/* Principal Amount Input */}
        <div>
          <label className="flex items-center text-xs uppercase tracking-wider text-gray-500">
            Principal Amount
            <InfoTooltip text="Initial amount to earn yield on" />
          </label>
          <div className="mt-2 flex items-center gap-2">
            <span className="text-sm text-gray-400">{currency}</span>
            <input
              type="number"
              min="0"
              step="100"
              value={principal}
              onChange={(e) => handleAmountChange(e.target.value)}
              className="w-full rounded-lg border border-white/10 bg-[#0A0F11] px-3 py-2 text-sm text-slate-100 outline-none transition-colors placeholder:text-slate-600 focus:border-primary"
            />
          </div>
        </div>

        {/* Holding Period Input */}
        <div>
          <label className="flex items-center text-xs uppercase tracking-wider text-gray-500">
            Holding Period
            <InfoTooltip text="Time to hold assets (1-50 years)" />
          </label>
          <div className="mt-2 flex items-center gap-2">
            <input
              type="number"
              min="1"
              max="50"
              value={years}
              onChange={(e) => handleYearsChange(e.target.value)}
              className="w-full rounded-lg border border-white/10 bg-[#0A0F11] px-3 py-2 text-sm text-slate-100 outline-none transition-colors placeholder:text-slate-600 focus:border-primary"
            />
            <span className="text-sm text-gray-400 whitespace-nowrap">years</span>
          </div>
        </div>
      </div>

      {/* Token Selection */}
      {showComparison && tokenRates.length > 1 && (
        <div>
          <label className="text-xs uppercase tracking-wider text-gray-500">Token Selection</label>
          <div className="mt-3 grid gap-2 sm:grid-cols-3">
            {tokenRates.map((token, index) => (
              <button
                key={token.name}
                type="button"
                onClick={() => setSelectedTokenIndex(index)}
                className={`rounded-lg border px-3 py-3 text-left transition-colors ${
                  selectedTokenIndex === index
                    ? 'border-primary bg-primary/10'
                    : 'border-white/10 bg-white/[0.02] hover:border-white/20'
                }`}
              >
                <p className="text-sm font-medium text-slate-100">{token.displayName}</p>
                <p className="text-xs text-gray-500 mt-0.5">
                  {(token.rateBps / 100).toFixed(2)}% APY
                </p>
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Yield Chart */}
      <YieldChart
        data={selectedCalculation.dataPoints}
        title={`Projected Growth - ${selectedToken.displayName}`}
        height={320}
        currency={currency}
      />

      {/* Summary Cards */}
      <div className="grid gap-4 sm:grid-cols-3">
        {/* Principal Card */}
        <div className="rounded-lg border border-white/10 bg-white/[0.02] p-4">
          <div className="flex items-center justify-between">
            <p className="text-xs uppercase tracking-wider text-gray-500">Principal</p>
            <Info size={14} className="text-gray-600" />
          </div>
          <p className="mt-2 text-xl font-bold text-slate-100">
            {currency}
            {principal.toLocaleString('en-US', {
              minimumFractionDigits: 2,
              maximumFractionDigits: 2,
            })}
          </p>
        </div>

        {/* Accrued Yield Card */}
        <div className="rounded-lg border border-white/10 bg-white/[0.02] p-4">
          <div className="flex items-center justify-between">
            <p className="text-xs uppercase tracking-wider text-gray-500">Accrued Yield</p>
            <TrendingUp size={14} className="text-emerald-400" />
          </div>
          <p className="mt-2 text-xl font-bold text-emerald-300">
            {currency}
            {selectedCalculation.finalAccruedYield.toLocaleString('en-US', {
              minimumFractionDigits: 2,
              maximumFractionDigits: 2,
            })}
          </p>
          <p className="mt-1 text-xs text-gray-500">
            {(selectedCalculation.ratePercentage / 100).toFixed(2)}% annual rate
          </p>
        </div>

        {/* Total Value Card */}
        <div className="rounded-lg border border-primary/30 bg-primary/5 p-4">
          <div className="flex items-center justify-between">
            <p className="text-xs uppercase tracking-wider text-gray-400">Final Total</p>
            <CheckCircleIcon />
          </div>
          <p className="mt-2 text-xl font-bold text-primary">
            {currency}
            {selectedCalculation.finalTotal.toLocaleString('en-US', {
              minimumFractionDigits: 2,
              maximumFractionDigits: 2,
            })}
          </p>
          <p className="mt-1 text-xs text-gray-500">
            +{((selectedCalculation.finalAccruedYield / principal) * 100).toFixed(1)}% return
          </p>
        </div>
      </div>

      {/* Information Box */}
      <div className="rounded-lg border border-blue-400/20 bg-blue-400/5 p-4">
        <p className="text-xs text-blue-300/80">
          <strong>Note:</strong> This calculator uses a simple interest formula that mirrors our backend
          yield calculations. Actual yields may vary based on market conditions and protocol changes.
        </p>
      </div>
    </div>
  );
}

/**
 * Simple check circle icon component
 */
function CheckCircleIcon() {
  return (
    <svg
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      className="text-primary"
    >
      <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
      <polyline points="22 4 12 14.01 9 11.01" />
    </svg>
  );
}
