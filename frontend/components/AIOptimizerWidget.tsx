'use client';

import { useEffect, useState } from 'react';
import { Zap, TrendingUp, Leaf, AlertCircle, Loader2, CheckCircle } from 'lucide-react';
import { getPlanOptimization, applyOptimization } from '@/lib/api/aiOptimization';
import type { PlanOptimizationResponse } from '@/lib/api/aiOptimization';

interface AIOptimizerWidgetProps {
  planId: string;
}

export function AIOptimizerWidget({ planId }: AIOptimizerWidgetProps) {
  const [optimization, setOptimization] = useState<PlanOptimizationResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [applying, setApplying] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [applied, setApplied] = useState(false);

  useEffect(() => {
    const loadOptimization = async () => {
      try {
        setLoading(true);
        const data = await getPlanOptimization(planId);
        setOptimization(data);
        setError(null);
      } catch (err) {
        setError('Failed to load optimization data');
        console.error(err);
      } finally {
        setLoading(false);
      }
    };

    loadOptimization();
  }, [planId]);

  const handleApplyOptimization = async () => {
    if (!optimization) return;

    try {
      setApplying(true);
      await applyOptimization(planId, optimization.recommendedParameters);
      setApplied(true);
      // Reset applied state after 3 seconds
      setTimeout(() => setApplied(false), 3000);
    } catch (err) {
      setError('Failed to apply optimization');
      console.error(err);
    } finally {
      setApplying(false);
    }
  };

  if (loading) {
    return (
      <div className="bg-gradient-to-br from-violet-900/20 to-blue-900/20 border border-violet-500/20 rounded-2xl p-6 space-y-4">
        <div className="flex items-center justify-center h-40">
          <Loader2 className="w-8 h-8 text-violet-400 animate-spin" />
        </div>
      </div>
    );
  }

  if (error || !optimization) {
    return (
      <div className="bg-gradient-to-br from-red-900/20 to-red-800/20 border border-red-500/20 rounded-2xl p-6">
        <div className="flex items-start gap-3">
          <AlertCircle className="w-5 h-5 text-red-400 flex-shrink-0 mt-1" />
          <div>
            <h3 className="text-sm font-semibold text-red-300">Unable to Load Recommendations</h3>
            <p className="text-xs text-red-400 mt-1">{error || 'Please try again later'}</p>
          </div>
        </div>
      </div>
    );
  }

  const healthScoreColor =
    optimization.healthScore >= 80
      ? 'text-emerald-400'
      : optimization.healthScore >= 60
      ? 'text-yellow-400'
      : 'text-red-400';

  const healthScoreBg =
    optimization.healthScore >= 80
      ? 'bg-emerald-500/10 border-emerald-500/20'
      : optimization.healthScore >= 60
      ? 'bg-yellow-500/10 border-yellow-500/20'
      : 'bg-red-500/10 border-red-500/20';

  return (
    <div className="space-y-4">
      {/* Main Health Score Card */}
      <div className={`border rounded-2xl p-6 bg-gradient-to-br from-violet-900/20 to-blue-900/20 border-violet-500/20`}>
        <div className="flex items-start justify-between mb-6">
          <div>
            <h2 className="text-lg font-semibold text-white mb-2 flex items-center gap-2">
              <Zap className="w-5 h-5 text-violet-400" />
              Plan Health Score
            </h2>
            <p className="text-xs text-slate-400">AI-powered optimization analysis</p>
          </div>
          <div className={`px-4 py-2 rounded-xl border flex items-center gap-2 ${healthScoreBg}`}>
            <span className={`text-2xl font-bold ${healthScoreColor}`}>
              {optimization.healthScore}
            </span>
            <span className={`text-xs font-semibold ${healthScoreColor}`}>/100</span>
          </div>
        </div>

        {/* Savings Summary */}
        <div className="grid grid-cols-3 gap-3 mb-6">
          <div className="bg-white/5 rounded-lg p-3 border border-white/10">
            <div className="text-xs text-slate-400 mb-1">Gas Savings</div>
            <div className="text-lg font-semibold text-emerald-400">
              {optimization.estimatedSavings.gasSavingsPercentage}%
            </div>
          </div>
          <div className="bg-white/5 rounded-lg p-3 border border-white/10">
            <div className="text-xs text-slate-400 mb-1">Tax Savings</div>
            <div className="text-lg font-semibold text-blue-400">
              ${optimization.estimatedSavings.taxSavingsUSD}
            </div>
          </div>
          <div className="bg-white/5 rounded-lg p-3 border border-white/10">
            <div className="text-xs text-slate-400 mb-1">Yield Increase</div>
            <div className="text-lg font-semibold text-yellow-400">
              {optimization.estimatedSavings.yieldIncreasePercentage}%
            </div>
          </div>
        </div>

        {/* Apply Button */}
        <button
          onClick={handleApplyOptimization}
          disabled={applying || applied}
          className={`w-full px-4 py-2.5 rounded-lg font-semibold text-sm transition-all flex items-center justify-center gap-2 ${
            applied
              ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30'
              : 'bg-violet-500 text-white hover:bg-violet-600 active:scale-95'
          } disabled:opacity-60 disabled:cursor-not-allowed`}
        >
          {applied ? (
            <>
              <CheckCircle className="w-4 h-4" />
              Applied Successfully
            </>
          ) : applying ? (
            <>
              <Loader2 className="w-4 h-4 animate-spin" />
              Applying...
            </>
          ) : (
            <>
              <Zap className="w-4 h-4" />
              Apply 1-Click Optimization
            </>
          )}
        </button>
      </div>

      {/* Gas Efficiency Suggestions */}
      <div className="border border-white/5 rounded-2xl p-4 bg-white/[0.02]">
        <h3 className="text-sm font-semibold text-white mb-3 flex items-center gap-2">
          <TrendingUp className="w-4 h-4 text-emerald-400" />
          Gas Efficiency
        </h3>
        <ul className="space-y-2">
          {optimization.gasEfficiencySuggestions.map((suggestion, idx) => (
            <li key={idx} className="text-xs text-slate-300 flex gap-2">
              <span className="text-emerald-400 font-bold flex-shrink-0">✓</span>
              <span>{suggestion}</span>
            </li>
          ))}
        </ul>
      </div>

      {/* Tax Loss Harvesting Tips */}
      <div className="border border-white/5 rounded-2xl p-4 bg-white/[0.02]">
        <h3 className="text-sm font-semibold text-white mb-3 flex items-center gap-2">
          <Leaf className="w-4 h-4 text-yellow-400" />
          Tax Optimization
        </h3>
        <ul className="space-y-2">
          {optimization.taxLossHarvestingTips.map((tip, idx) => (
            <li key={idx} className="text-xs text-slate-300 flex gap-2">
              <span className="text-yellow-400 font-bold flex-shrink-0">→</span>
              <span>{tip}</span>
            </li>
          ))}
        </ul>
      </div>

      {/* Yield Enhancement */}
      <div className="border border-white/5 rounded-2xl p-4 bg-white/[0.02]">
        <h3 className="text-sm font-semibold text-white mb-3 flex items-center gap-2">
          <TrendingUp className="w-4 h-4 text-blue-400" />
          Yield Enhancement
        </h3>
        <ul className="space-y-2">
          {optimization.yieldEnhancementRecommendations.map((recommendation, idx) => (
            <li key={idx} className="text-xs text-slate-300 flex gap-2">
              <span className="text-blue-400 font-bold flex-shrink-0">+</span>
              <span>{recommendation}</span>
            </li>
          ))}
        </ul>
      </div>

      {/* Recommended Parameters */}
      <div className="bg-gradient-to-br from-slate-900/50 to-slate-800/50 border border-slate-700/50 rounded-2xl p-4">
        <h3 className="text-sm font-semibold text-white mb-3">Recommended Parameters</h3>
        <div className="grid grid-cols-2 gap-3 text-xs">
          {optimization.recommendedParameters.beneficiaryCount && (
            <div className="bg-white/5 rounded-lg p-2 border border-white/10">
              <div className="text-slate-400">Beneficiary Count</div>
              <div className="font-semibold text-white mt-1">
                {optimization.recommendedParameters.beneficiaryCount}
              </div>
            </div>
          )}
          {optimization.recommendedParameters.distributionFrequency && (
            <div className="bg-white/5 rounded-lg p-2 border border-white/10">
              <div className="text-slate-400">Distribution</div>
              <div className="font-semibold text-white mt-1 capitalize">
                {optimization.recommendedParameters.distributionFrequency}
              </div>
            </div>
          )}
          {optimization.recommendedParameters.anchorType && (
            <div className="bg-white/5 rounded-lg p-2 border border-white/10">
              <div className="text-slate-400">Anchor Type</div>
              <div className="font-semibold text-white mt-1 capitalize">
                {optimization.recommendedParameters.anchorType}
              </div>
            </div>
          )}
          {optimization.recommendedParameters.riskProfile && (
            <div className="bg-white/5 rounded-lg p-2 border border-white/10">
              <div className="text-slate-400">Risk Profile</div>
              <div className="font-semibold text-white mt-1 capitalize">
                {optimization.recommendedParameters.riskProfile}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
