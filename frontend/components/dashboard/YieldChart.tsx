'use client';

import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
  TooltipProps,
} from 'recharts';
import { YieldDataPoint } from '@/app/hooks/useYieldCalculations';

interface YieldChartProps {
  data: YieldDataPoint[];
  title?: string;
  height?: number;
  currency?: string;
}

/**
 * Custom tooltip that shows detailed yield breakdown
 */
function YieldTooltip({
  active,
  payload,
  currency = '$',
}: any) {
  if (!active || !payload || payload.length === 0) return null;

  const data = payload[0].payload as YieldDataPoint;

  return (
    <div className="rounded-lg border border-white/20 bg-slate-900/95 p-3 shadow-lg backdrop-blur">
      <p className="text-xs text-gray-400 mb-2">
        Year {data.year}, Month {data.month % 12}
      </p>
      <div className="space-y-1.5">
        <div className="flex justify-between gap-4">
          <span className="text-xs text-gray-300">Principal:</span>
          <span className="text-sm font-semibold text-slate-100">
            {currency}
            {data.principal.toLocaleString('en-US', {
              minimumFractionDigits: 2,
              maximumFractionDigits: 2,
            })}
          </span>
        </div>
        <div className="flex justify-between gap-4">
          <span className="text-xs text-gray-300">Accrued Yield:</span>
          <span className="text-sm font-semibold text-emerald-300">
            {currency}
            {data.accruedYield.toLocaleString('en-US', {
              minimumFractionDigits: 2,
              maximumFractionDigits: 2,
            })}
          </span>
        </div>
        <div className="border-t border-white/10 pt-1.5 flex justify-between gap-4">
          <span className="text-xs text-gray-300">Total:</span>
          <span className="text-sm font-bold text-primary">
            {currency}
            {data.total.toLocaleString('en-US', {
              minimumFractionDigits: 2,
              maximumFractionDigits: 2,
            })}
          </span>
        </div>
        <p className="text-xs text-gray-500 mt-2 pt-1.5 border-t border-white/10">
          Yield Rate: {(data.yieldRate * 100).toFixed(2)}% APY
        </p>
      </div>
    </div>
  );
}

export function YieldChart({
  data,
  title = 'Projected Yield Over Time',
  height = 300,
  currency = '$',
}: YieldChartProps) {
  if (data.length === 0) {
    return (
      <div className="w-full h-80 flex items-center justify-center rounded-lg border border-white/10 bg-white/[0.03]">
        <p className="text-sm text-gray-500">No data to display</p>
      </div>
    );
  }

  return (
    <div className="space-y-3">
      {title && <h3 className="text-sm font-semibold text-slate-100">{title}</h3>}
      <div className="rounded-lg border border-white/10 bg-white/[0.03] p-4">
        <ResponsiveContainer width="100%" height={height}>
          <LineChart data={data} margin={{ top: 5, right: 30, left: 0, bottom: 5 }}>
            <CartesianGrid 
              strokeDasharray="3 3" 
              stroke="rgba(255,255,255,0.1)"
              vertical={false}
            />
            <XAxis
              dataKey="year"
              stroke="rgba(255,255,255,0.5)"
              style={{ fontSize: '12px' }}
              label={{ value: 'Year', position: 'insideBottomRight', offset: -5 }}
            />
            <YAxis
              stroke="rgba(255,255,255,0.5)"
              style={{ fontSize: '12px' }}
              label={{ 
                value: `Amount (${currency})`,
                angle: -90,
                position: 'insideLeft',
              }}
              tickFormatter={(value) =>
                `${currency}${(value / 1000).toFixed(0)}k`
              }
            />
            <Tooltip content={<YieldTooltip currency={currency} />} />
            <Legend wrapperStyle={{ paddingTop: '16px' }} />
            <Line
              type="monotone"
              dataKey="principal"
              stroke="rgba(94, 109, 241, 0.7)"
              strokeWidth={2}
              dot={false}
              isAnimationActive={false}
              name="Principal"
            />
            <Line
              type="monotone"
              dataKey="total"
              stroke="rgba(34, 197, 94, 0.8)"
              strokeWidth={2}
              dot={false}
              isAnimationActive={false}
              name="Total (Principal + Yield)"
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
