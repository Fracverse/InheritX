"use client";
import { PieChart, Pie, Cell, Tooltip, ResponsiveContainer, Legend } from "recharts";

const assets = [
  { name: "USDC", value: 62, color: "#33c5e0" },
  { name: "XLM", value: 28, color: "#818cf8" },
  { name: "Other", value: 10, color: "#f59e0b" },
];

const CustomTooltip = ({ active, payload }: any) => {
  if (!active || !payload?.length) return null;
  return (
    <div className="rounded-lg border border-white/10 bg-[#0f0f0f] px-3 py-2 text-xs shadow-xl">
      <p className="text-foreground/50">{payload[0].name}</p>
      <p className="font-semibold" style={{ color: payload[0].payload.color }}>
        {payload[0].value}%
      </p>
    </div>
  );
};

const renderLegend = () => (
  <div className="flex flex-col gap-2 mt-4">
    {assets.map((a) => (
      <div key={a.name} className="flex items-center justify-between text-xs">
        <div className="flex items-center gap-2">
          <span className="w-2.5 h-2.5 rounded-full" style={{ background: a.color }} />
          <span className="text-foreground/60">{a.name}</span>
        </div>
        <span className="font-semibold tabular-nums" style={{ color: a.color }}>
          {a.value}%
        </span>
      </div>
    ))}
  </div>
);

export function AssetDistributionChart() {
  return (
    <div className="rounded-xl border border-white/5 bg-white/[0.02] sm:p-6 p-4 h-full">
      <div className="mb-5">
        <h2 className="text-sm font-semibold text-foreground">Asset Distribution</h2>
        <p className="text-xs text-foreground/40 mt-0.5">By locked value</p>
      </div>
      <ResponsiveContainer width="100%" height={200}>
        <PieChart>
          <Pie data={assets} cx="50%" cy="50%" innerRadius={55} outerRadius={85} paddingAngle={3} dataKey="value" strokeWidth={0}>
            {assets.map((entry, i) => (
              <Cell key={i} fill={entry.color} />
            ))}
          </Pie>
          <Tooltip content={<CustomTooltip />} />
        </PieChart>
      </ResponsiveContainer>
      {renderLegend()}
    </div>
  );
}
