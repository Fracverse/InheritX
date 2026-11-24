'use client';

import { useCallback, useEffect, useState } from 'react';
import { Zap, TrendingUp, AlertCircle, Loader2, CheckCircle } from 'lucide-react';
import { getPlanOptimization, applyOptimization } from '@/lib/api/aiOptimization';
import type {
  OptimizationRecommendation,
  RecommendationResponse,
} from '@/lib/api/aiOptimization';

interface AIOptimizerWidgetProps {
  planId: string;
}

function errorMessage(err: unknown, fallback: string): string {
  return err instanceof Error && err.message ? err.message : fallback;
}

export function AIOptimizerWidget({ planId }: AIOptimizerWidgetProps) {
  const [optimization, setOptimization] = useState<OptimizationRecommendation | null>(null);
  const [loading, setLoading] = useState(true);
  const [applying, setApplying] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [applyError, setApplyError] = useState<string | null>(null);
  const [confirmation, setConfirmation] = useState<RecommendationResponse | null>(null);

  useEffect(() => {
    let cancelled = false;
    const loadOptimization = async () => {
      try {
        setLoading(true);
        setConfirmation(null);
        setApplyError(null);
        const data = await getPlanOptimization(planId);
        if (cancelled) return;
        setOptimization(data);
        setError(null);
      } catch (err) {
        if (cancelled) return;
        setOptimization(null);
        setError(errorMessage(err, 'Failed to load optimization data'));
      } finally {
        if (!cancelled) setLoading(false);
      }
    };

    loadOptimization();
    return () => {
      cancelled = true;
    };
  }, [planId]);

  const handleApplyOptimization = useCallback(async () => {
    if (!optimization) return;

    try {
      setApplying(true);
      setApplyError(null);
      const result = await applyOptimization(optimization.id);
      setConfirmation(result);
      // Reflect server-confirmed plan state: reload from the server rather
      // than assuming the recommended values were applied.
      const refreshed = await getPlanOptimization(planId);
      setOptimization(refreshed);
    } catch (err) {
      setApplyError(errorMessage(err, 'Failed to apply optimization'));
    } finally {
      setApplying(false);
    }
  }, [optimization, planId]);

  if (loading) {
    return (
      <div role="status" aria-label="Loading recommendations" className="bg-gradient-to-br from-violet-900/20 to-blue-900/20 border border-violet-500/20 rounded-2xl p-6 space-y-4">
        <div className="flex items-center justify-center h-40">
          <Loader2 className="w-8 h-8 text-violet-400 animate-spin" />
        </div>
      </div>
    );
  }

  if (error || !optimization) {
    return (
      <div role="alert" className="bg-gradient-to-br from-red-900/20 to-red-800/20 border border-red-500/20 rounded-2xl p-6">
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

  if (optimization.recommendedAllocations.length === 0) {
    return (
      <div className="border border-white/5 rounded-2xl p-6 bg-white/[0.02]">
        <h3 className="text-sm font-semibold text-white mb-1">No Recommendations Available</h3>
        <p className="text-xs text-slate-400">
          There are no optimization recommendations for this plan right now.
        </p>
      </div>
    );
  }

  const applied = confirmation?.status === 'accepted';

  return (
    <div className="space-y-4">
      <div className="border rounded-2xl p-6 bg-gradient-to-br from-violet-900/20 to-blue-900/20 border-violet-500/20">
        <div className="mb-6">
          <h2 className="text-lg font-semibold text-white mb-2 flex items-center gap-2">
            <Zap className="w-5 h-5 text-violet-400" />
            AI Optimization
          </h2>
          <p className="text-xs text-slate-400">{optimization.reasoning}</p>
        </div>

        <div className="grid grid-cols-3 gap-3 mb-6">
          <div className="bg-white/5 rounded-lg p-3 border border-white/10">
            <div className="text-xs text-slate-400 mb-1">Confidence</div>
            <div className="text-lg font-semibold text-emerald-400">{optimization.confidenceScore}</div>
          </div>
          <div className="bg-white/5 rounded-lg p-3 border border-white/10">
            <div className="text-xs text-slate-400 mb-1">Expected Return</div>
            <div className="text-lg font-semibold text-blue-400">{optimization.expectedReturn}%</div>
          </div>
          <div className="bg-white/5 rounded-lg p-3 border border-white/10">
            <div className="text-xs text-slate-400 mb-1">Risk Score</div>
            <div className="text-lg font-semibold text-yellow-400">{optimization.riskScore}</div>
          </div>
        </div>

        {applyError && (
          <p role="alert" className="text-xs text-red-400 mb-3">
            {applyError}
          </p>
        )}

        {applied && confirmation && (
          <p role="status" className="text-xs text-emerald-400 mb-3">
            Accepted by the server at {new Date(confirmation.appliedAt).toLocaleString()}. Allocations below are the plan&apos;s current values from the server.
          </p>
        )}

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
              Applied
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

      <div className="border border-white/5 rounded-2xl p-4 bg-white/[0.02]">
        <h3 className="text-sm font-semibold text-white mb-3 flex items-center gap-2">
          <TrendingUp className="w-4 h-4 text-emerald-400" />
          Allocations
        </h3>
        <ul className="space-y-3">
          {optimization.recommendedAllocations.map((a) => (
            <li key={`${a.chain}-${a.assetSymbol}`} className="text-xs text-slate-300">
              <div className="flex justify-between">
                <span className="font-semibold text-white">
                  {a.assetSymbol} <span className="text-slate-400">({a.chain})</span>
                </span>
                <span>
                  {a.currentPercentage}% → {a.recommendedPercentage}%
                </span>
              </div>
              <div className="text-slate-400 mt-1">{a.adjustmentReason}</div>
              <div className="text-slate-400">{a.expectedImpact}</div>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
