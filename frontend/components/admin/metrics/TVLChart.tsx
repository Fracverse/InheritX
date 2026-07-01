"use client";
import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from "recharts";

// Mock TVL time-series until backend exposes a history endpoint
const tvlData = [
  { month: "Jan", tvl: 120000 },
  { month: "Feb", tvl: 185000 },
  { month: "Mar", tvl: 162000 },
  { month: "Apr", tvl: 240000 },
  { month: "May", tvl: 310000 },
  { month: "Jun", tvl: 295000 },
  { month: "Jul", tvl: 420000 },
  { month: "Aug", tvl: 510000 },
  { month: "Sep", tvl: 480000 },
  { month: "Oct", tvl: 590000 },
  { month: "Nov", tvl: 670000 },
  { month: "Dec", tvl: 720000 },
];

const CustomTooltip = ({ active, payload, label }: any) => {
  if (!active || !payload?.length) return null;
  return (
    <div className="rounded-lg border border-white/10 bg-[#0f0f0f] px-3 py-2 text-xs shadow-xl">
      <p className="text-foreground/50 mb-1">{label}</p>
      <p className="text-primary font-semibold">${payload[0].value.toLocaleString()}</p>
    </div>
  );
};

export function TVLChart() {
  return (
    <div className="rounded-xl border border-white/5 bg-white/[0.02] sm:p-6 p-4 h-full">
      <div className="mb-5">
        <h2 className="text-sm font-semibold text-foreground">Total Value Locked (TVL)</h2>
        <p className="text-xs text-foreground/40 mt-0.5">12-month overview</p>
      </div>
      <ResponsiveContainer width="100%" height={240}>
        <AreaChart data={tvlData} margin={{ top: 4, right: 4, left: 0, bottom: 0 }}>
          <defs>
            <linearGradient id="tvlGradient" x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor="#33c5e0" stopOpacity={0.2} />
              <stop offset="95%" stopColor="#33c5e0" stopOpacity={0} />
            </linearGradient>
          </defs>
          <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.04)" />
          <XAxis dataKey="month" tick={{ fill: "rgba(237,237,237,0.4)", fontSize: 11 }} axisLine={false} tickLine={false} />
          <YAxis tick={{ fill: "rgba(237,237,237,0.4)", fontSize: 11 }} axisLine={false} tickLine={false} tickFormatter={(v) => `$${(v / 1000).toFixed(0)}k`} />
          <Tooltip content={<CustomTooltip />} />
          <Area
            type="monotone"
            dataKey="tvl"
            stroke="#33c5e0"
            strokeWidth={2}
            fill="url(#tvlGradient)"
            dot={false}
            activeDot={{ r: 4, fill: "#33c5e0", strokeWidth: 0 }}
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
}
