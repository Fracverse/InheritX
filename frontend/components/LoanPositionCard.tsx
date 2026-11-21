'use client';

import React from 'react';
import { AlertTriangle, TrendingUp } from 'lucide-react';
import clsx from 'clsx';

interface LoanPosition {
  loanId: string;
  nftId: string;
  collateralAmount: number;
  borrowAmount: number;
  collateralSymbol: string;
  borrowSymbol: string;
  collateralPrice: number;
  liquidationThreshold: number; // e.g., 80 (80%)
}

interface LoanPositionCardProps {
  position: LoanPosition;
  onManage?: () => void;
  onRepay?: () => void;
}

/**
 * LoanPositionCard displays NFT-backed loan positions with LTV, health factor,
 * and visual warnings when liquidation risk is high.
 *
 * Features:
 * - Current LTV calculation and display
 * - Liquidation threshold indicator (80%)
 * - Visual warning when LTV > 75%
 * - Loan NFT ID badge
 * - Quick action buttons (Manage, Repay)
 */
export function LoanPositionCard({
  position,
  onManage,
  onRepay,
}: LoanPositionCardProps) {
  // Calculate LTV (Loan-To-Value)
  // LTV = Borrow Amount / Collateral Value
  const collateralValue = position.collateralAmount * position.collateralPrice;
  const ltv = collateralValue > 0 ? (position.borrowAmount / collateralValue) * 100 : 0;

  // Check if LTV exceeds warning threshold (75%)
  const warningThreshold = 75;
  const isWarning = ltv > warningThreshold;
  const isNearLiquidation = ltv > position.liquidationThreshold;

  // Calculate health factor
  // Health Factor = (Collateral Value × LiquidationThreshold) / Borrow Amount
  const healthFactor =
    position.borrowAmount > 0
      ? (collateralValue * (position.liquidationThreshold / 100)) / position.borrowAmount
      : Infinity;

  // Determine health status
  const getHealthStatus = () => {
    if (healthFactor === Infinity) return { label: 'Healthy', color: 'text-green-600' };
    if (healthFactor > 1.5) return { label: 'Healthy', color: 'text-green-600' };
    if (healthFactor > 1.2) return { label: 'Caution', color: 'text-yellow-600' };
    return { label: 'Risk', color: 'text-red-600' };
  };

  const healthStatus = getHealthStatus();

  return (
    <div
      className={clsx(
        'rounded-lg border p-6 transition-all',
        isNearLiquidation
          ? 'border-red-300 bg-red-50 shadow-sm shadow-red-100'
          : isWarning
            ? 'border-yellow-300 bg-yellow-50 shadow-sm shadow-yellow-100'
            : 'border-gray-200 bg-white shadow-sm'
      )}
    >
      {/* Header with warnings */}
      <div className="mb-4 flex items-start justify-between">
        <div>
          <h3 className="flex items-center gap-2 text-lg font-semibold">
            Loan Position
            {isWarning && (
              <AlertTriangle className="h-5 w-5 text-yellow-600" />
            )}
          </h3>
          <p className="text-sm text-gray-600">NFT ID: {position.nftId}</p>
        </div>
        <div className="rounded-full bg-gray-100 px-3 py-1">
          <span className={clsx('text-sm font-medium', healthStatus.color)}>
            {healthStatus.label}
          </span>
        </div>
      </div>

      {/* Main metrics grid */}
      <div className="mb-6 grid grid-cols-2 gap-4 md:grid-cols-4">
        {/* Current LTV */}
        <div className="rounded-lg bg-gray-50 p-3">
          <p className="text-xs text-gray-600">Current LTV</p>
          <p className={clsx('text-xl font-bold', isWarning ? 'text-yellow-600' : 'text-gray-900')}>
            {ltv.toFixed(1)}%
          </p>
          <p className="text-xs text-gray-500">of {position.liquidationThreshold}% max</p>
        </div>

        {/* Liquidation Threshold */}
        <div className="rounded-lg bg-gray-50 p-3">
          <p className="text-xs text-gray-600">Liquidation Threshold</p>
          <p className="text-xl font-bold text-gray-900">{position.liquidationThreshold}%</p>
          <p className="text-xs text-red-600">safety limit</p>
        </div>

        {/* Health Factor */}
        <div className="rounded-lg bg-gray-50 p-3">
          <p className="text-xs text-gray-600">Health Factor</p>
          <p
            className={clsx(
              'text-xl font-bold',
              healthFactor === Infinity ? 'text-green-600' : healthStatus.color
            )}
          >
            {healthFactor === Infinity ? '∞' : healthFactor.toFixed(2)}
          </p>
          <p className="text-xs text-gray-500">min 1.0 safe</p>
        </div>

        {/* Borrow Amount */}
        <div className="rounded-lg bg-gray-50 p-3">
          <p className="text-xs text-gray-600">You Owe</p>
          <p className="text-xl font-bold text-gray-900">
            {position.borrowAmount.toLocaleString('en-US', {
              minimumFractionDigits: 2,
              maximumFractionDigits: 2,
            })}
          </p>
          <p className="text-xs text-gray-500">{position.borrowSymbol}</p>
        </div>
      </div>

      {/* LTV Gauge */}
      <div className="mb-6">
        <div className="mb-2 flex items-center justify-between">
          <p className="text-sm font-medium text-gray-700">LTV Progress</p>
          <p className="text-xs text-gray-600">
            {position.borrowAmount.toLocaleString('en-US', {
              minimumFractionDigits: 2,
              maximumFractionDigits: 2,
            })}{' '}
            / {collateralValue.toLocaleString('en-US', {
              minimumFractionDigits: 2,
              maximumFractionDigits: 2,
            })}
          </p>
        </div>
        <div className="h-3 w-full overflow-hidden rounded-full bg-gray-200">
          <div
            className={clsx('h-full transition-all', isNearLiquidation ? 'bg-red-600' : isWarning ? 'bg-yellow-500' : 'bg-green-500')}
            style={{ width: `${Math.min(ltv, 100)}%` }}
          />
        </div>
        <div className="mt-2 flex justify-between text-xs text-gray-600">
          <span>0%</span>
          <span>{warningThreshold}% (Warning)</span>
          <span>{position.liquidationThreshold}% (Liquidation)</span>
        </div>
      </div>

      {/* Collateral Info */}
      <div className="mb-6 rounded-lg bg-blue-50 p-3">
        <div className="flex items-start justify-between">
          <div>
            <p className="text-xs text-blue-600">Collateral</p>
            <p className="text-lg font-semibold text-gray-900">
              {position.collateralAmount.toLocaleString('en-US', {
                minimumFractionDigits: 2,
                maximumFractionDigits: 2,
              })}{' '}
              {position.collateralSymbol}
            </p>
            <p className="text-xs text-gray-600">
              ≈ $
              {collateralValue.toLocaleString('en-US', {
                minimumFractionDigits: 2,
                maximumFractionDigits: 2,
              })}
            </p>
          </div>
          <TrendingUp className="h-5 w-5 text-blue-600" />
        </div>
      </div>

      {/* Action buttons */}
      <div className="flex gap-2">
        <button
          onClick={onRepay}
          className="flex-1 rounded-lg bg-blue-600 px-4 py-2 font-medium text-white transition-colors hover:bg-blue-700"
        >
          Repay Loan
        </button>
        <button
          onClick={onManage}
          className="flex-1 rounded-lg border border-gray-300 px-4 py-2 font-medium text-gray-700 transition-colors hover:bg-gray-50"
        >
          Manage
        </button>
      </div>

      {/* Warning messages */}
      {isWarning && (
        <div className="mt-4 rounded-lg bg-yellow-50 p-3">
          <p className="text-sm text-yellow-800">
            ⚠️ <strong>High LTV Warning:</strong> Your loan is approaching the liquidation threshold. Consider repaying or adding collateral.
          </p>
        </div>
      )}

      {isNearLiquidation && (
        <div className="mt-4 rounded-lg bg-red-50 p-3">
          <p className="text-sm text-red-800">
            🚨 <strong>Critical:</strong> Your loan is at risk of liquidation. Immediate action required.
          </p>
        </div>
      )}
    </div>
  );
}

export default LoanPositionCard;
