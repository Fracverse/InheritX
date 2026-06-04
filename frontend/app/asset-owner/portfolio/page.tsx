import React from "react";
import { ChevronDown } from "lucide-react";
import { ArrowTopRightOnSquareIcon } from "@heroicons/react/24/outline";

function Portfolio() {
  const chartData = [
    { month: "Jan", value: 30 },
    { month: "Feb", value: 45 },
    { month: "Mar", value: 28 },
    { month: "Apr", value: 60 },
    { month: "May", value: 42 },
    { month: "Jun", value: 75 },
    { month: "Jul", value: 55 },
    { month: "Aug", value: 68 },
    { month: "Sep", value: 52 },
    { month: "Oct", value: 80 },
    { month: "Nov", value: 65 },
    { month: "Dec", value: 90 },
  ];

  const assets = [
    { name: "ETH", amount: "2.45", price: "$3,500", value: "$7,640" },
    { name: "ETH", amount: "2.45", price: "$3,500", value: "$7,640" },
    { name: "ETH", amount: "2.45", price: "$3,500", value: "$7,640" },
    { name: "ETH", amount: "2.45", price: "$3,500", value: "$7,640" },
  ];

  return (
    <div className="min-h-screen text-white p-4 sm:p-6">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-2xl sm:text-3xl font-semibold mb-1">Portfolio</h1>
        <p className="text-gray-400 text-sm sm:text-base">
          Click on the display of your wealth so far.
        </p>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-12">
        {/* Card 1 */}
        <div className="bg-gray-800/40 rounded-lg p-6 border border-gray-700/50 hover:border-gray-600 transition-colors">
          <div className="flex items-start justify-between mb-1">
            <div className="text-4xl font-bold">0</div>
            <div className="flex gap-1">
              <div className="w-2 h-8 bg-green-500 rounded-sm"></div>
              <div className="w-2 h-6 bg-green-400 rounded-sm mt-2"></div>
              <div className="w-2 h-4 bg-green-300 rounded-sm mt-4"></div>
            </div>
          </div>
          <div className="text-gray-400 text-sm mb-3">
            Total Portfolio Value
          </div>
          <button className="border border-gray-700 rounded-full px-3 py-1.5 text-gray-400 text-sm hover:bg-gray-800 hover:text-gray-300 transition-all">
            See Asset Cycle Tree
          </button>
        </div>

        {/* Card 2 */}
        <div className="bg-gray-800/40 rounded-lg p-6 border border-gray-700/50 hover:border-gray-600 transition-colors">
          <div className="text-4xl font-bold mb-1">0</div>
          <div className="text-gray-400 text-sm mb-1">Opened Count</div>
          <div className="text-gray-500 text-xs mb-3">
            Equals 15% of activity
          </div>
          <button className="border border-gray-700 rounded-full px-3 py-1.5 text-gray-400 text-sm hover:bg-gray-800 hover:text-gray-300 transition-all">
            My Asset Count Pie
          </button>
        </div>

        {/* Card 3 */}
        <div className="bg-gray-800/40 rounded-lg p-6 border border-gray-700/50 hover:border-gray-600 transition-colors">
          <div className="text-4xl font-bold mb-1">0</div>
          <div className="text-gray-400 text-sm mb-3">NFT count</div>
          <button className="border border-gray-700 rounded-full px-3 py-1.5 text-gray-400 text-sm hover:bg-gray-800 hover:text-gray-300 transition-all">
            <div className="flex -space-x-2">
              <div className="w-6 h-6 rounded-full bg-blue-500 border-2 border-gray-800"></div>
              <div className="w-6 h-6 rounded-full bg-orange-500 border-2 border-gray-800"></div>
              <div className="w-6 h-6 rounded-full bg-purple-500 border-2 border-gray-800"></div>
              <div className="w-6 h-6 rounded-full bg-pink-500 border-2 border-gray-800"></div>
            </div>
          </button>
        </div>

        {/* Card 4 */}
        <div className="bg-gray-800/40 rounded-lg p-6 border border-gray-700/50 hover:border-gray-600 transition-colors">
          <div className="flex items-center gap-3 mb-1">
            <div className="text-4xl font-bold">+ $1,250</div>
          </div>
          <div className="text-gray-400 text-sm mb-1">Recent Images</div>
          <div className="text-gray-500 text-xs mb-3">This week</div>
          <button className="border border-gray-700 rounded-full px-3 py-1.5 text-gray-400 text-sm hover:bg-gray-800 hover:text-gray-300 transition-all flex items-center gap-2">
            Test images
            <ChevronDown size={16} />
          </button>
        </div>
      </div>

      {/* Charts & Insights Section */}
      <div className="mb-9">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-xl sm:text-2xl font-semibold">
            CHART & INSIGHTS
          </h2>
          <button className="text-gray-400 hover:text-gray-300">
            <ChevronDown size={20} />
          </button>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* Line Chart */}
          <div className="lg:col-span-1 bg-gray-800/40 rounded-lg p-6 border border-gray-700/50">
            <div className="relative">
              {/* Y-axis label */}
              <div className="absolute -top-2 left-0 text-cyan-400 text-xs">
                (Value)
              </div>

              {/* Y-axis values */}
              <div className="absolute left-0 top-6 flex flex-col justify-between h-52 text-gray-400 text-xs">
                <div>100</div>
                <div>75</div>
                <div>50</div>
                <div>25</div>
                <div>0</div>
              </div>

              {/* Chart area */}
              <div className="ml-8 relative">
                <svg
                  className="w-full h-64"
                  viewBox="0 0 800 260"
                  preserveAspectRatio="xMidYMid meet"
                >
                  {/* Vertical grid lines */}
                  {[0, 1, 2, 3, 4, 5, 6, 7].map((i) => (
                    <line
                      key={`v-${i}`}
                      x1={i * 100 + 50}
                      y1="10"
                      x2={i * 100 + 50}
                      y2="210"
                      stroke="rgb(55, 65, 81)"
                      strokeWidth="1"
                      opacity="0.3"
                    />
                  ))}

                  {/* Horizontal grid lines */}
                  {[0, 1, 2, 3, 4].map((i) => (
                    <line
                      key={`h-${i}`}
                      x1="0"
                      y1={i * 50 + 10}
                      x2="800"
                      y2={i * 50 + 10}
                      stroke="rgb(55, 65, 81)"
                      strokeWidth="1"
                      opacity="0.3"
                    />
                  ))}

                  {/* Line path - matching the exact pattern from the image */}
                  <path
                    d="M 50 150 L 150 30 L 250 90 L 350 95 L 450 25 L 550 155 L 650 145 L 750 20"
                    fill="none"
                    stroke="rgb(6, 182, 212)"
                    strokeWidth="2"
                  />

                  {/* Data points */}
                  {[
                    { x: 50, y: 150 },
                    { x: 150, y: 30 },
                    { x: 250, y: 90 },
                    { x: 350, y: 95 },
                    { x: 450, y: 25 },
                    { x: 550, y: 155 },
                    { x: 650, y: 145 },
                    { x: 750, y: 20 },
                  ].map((point, i) => (
                    <circle
                      key={i}
                      cx={point.x}
                      cy={point.y}
                      r="4"
                      fill="rgb(6, 182, 212)"
                    />
                  ))}
                </svg>

                {/* X-axis labels */}
                <div className="flex justify-between px-6 text-xs text-gray-400 mt-2">
                  {[0, 1, 2, 3, 4, 5, 6, 7].map((num) => (
                    <div key={num} className="w-12 text-center">
                      {num}
                    </div>
                  ))}
                </div>

                {/* X-axis label */}
                <div className="text-right text-cyan-400 text-xs mt-2 pr-6">
                  (Days)
                </div>
              </div>
            </div>

            {/* Period buttons */}
            <div className="flex gap-2 mt-6 ml-8">
              {["1H", "1D", "1W", "1M", "1Y"].map((period, index) => (
                <button
                  key={period}
                  className={`px-4 py-1.5 rounded text-xs font-medium transition-colors ${
                    index === 0
                      ? "bg-cyan-500 text-white"
                      : "bg-gray-700/50 text-gray-400 hover:bg-gray-700"
                  }`}
                >
                  {period}
                </button>
              ))}
            </div>
          </div>

          {/* Donut Chart */}
          <div className="bg-gray-800/40 rounded-lg p-6 border border-gray-700/50">
            <div className="mb-4">
              <div className="text-sm text-gray-400 mb-1">
                Total asset pool value
              </div>
              <div className="text-xs text-gray-500">
                Total of all asset Values
              </div>
            </div>

            <div className="flex items-center justify-center mb-6">
              <div className="relative">
                <svg width="180" height="180" viewBox="0 0 180 180">
                  {/* Orange segment */}
                  <circle
                    cx="90"
                    cy="90"
                    r="65"
                    fill="none"
                    stroke="rgb(249, 115, 22)"
                    strokeWidth="24"
                    strokeDasharray="153 410"
                    transform="rotate(-90 90 90)"
                  />
                  {/* Purple segment */}
                  <circle
                    cx="90"
                    cy="90"
                    r="65"
                    fill="none"
                    stroke="rgb(147, 51, 234)"
                    strokeWidth="24"
                    strokeDasharray="102 410"
                    strokeDashoffset="-153"
                    transform="rotate(-90 90 90)"
                  />
                  {/* Pink segment */}
                  <circle
                    cx="90"
                    cy="90"
                    r="65"
                    fill="none"
                    stroke="rgb(236, 72, 153)"
                    strokeWidth="24"
                    strokeDasharray="51 410"
                    strokeDashoffset="-255"
                    transform="rotate(-90 90 90)"
                  />
                </svg>

                {/* Center text */}
                <div className="absolute inset-0 flex flex-col items-center justify-center">
                  <div className="text-gray-400 text-xs">80%</div>
                  <div className="text-white text-lg font-bold">$25</div>
                  <div className="text-gray-400 text-xs">38%</div>
                </div>
              </div>
            </div>

            <div className="space-y-3 text-xs">
              <div>
                <div className="flex items-center justify-between mb-1">
                  <div className="flex items-center gap-2">
                    <div className="w-2 h-2 rounded-full bg-orange-500"></div>
                    <span className="text-gray-400">New World Asset</span>
                  </div>
                  <span className="text-white">48%</span>
                </div>
              </div>

              <div>
                <div className="flex items-center justify-between mb-1">
                  <div className="flex items-center gap-2">
                    <div className="w-2 h-2 rounded-full bg-purple-600"></div>
                    <span className="text-gray-400">ETH</span>
                  </div>
                  <span className="text-white">32%</span>
                </div>
              </div>

              <div>
                <div className="flex items-center justify-between mb-1">
                  <div className="flex items-center gap-2">
                    <div className="w-2 h-2 rounded-full bg-pink-500"></div>
                    <span className="text-gray-400">USDT</span>
                  </div>
                  <span className="text-white">15%</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Asset Table */}
      <div className="mb-9">
        <h2 className="text-xl sm:text-2xl font-semibold mb-6">YOUR ASSETS</h2>

        <div className="bg-gray-800/40 rounded-lg border border-gray-700/50 overflow-hidden">
          {/* Table Header */}
          <div className="grid grid-cols-[1.2fr_1fr_1fr_1fr_1.8fr_0.3fr] gap-4 p-4 px-6 border-b border-gray-700/50 text-xs text-gray-500 font-normal uppercase">
            <div>Assets</div>
            <div className="text-center">Balance</div>
            <div className="text-center">Price ($ USD)</div>
            <div className="text-center">Value</div>
            <div className="text-center">Action</div>
            <div></div>
          </div>

          {/* Table Rows */}
          {assets.map((asset, index) => (
            <div
              key={index}
              className="grid grid-cols-[1.2fr_1fr_1fr_1fr_1.8fr_0.3fr] gap-4 p-4 px-6 border-b border-gray-700/30 items-center"
            >
              {/* Asset */}
              <div className="flex items-center gap-3">
                <div className="w-7 h-7 rounded-full bg-gray-800 flex items-center justify-center">
                  <div className="w-4 h-4 rounded-full border-2 border-gray-400"></div>
                </div>
                <span className="text-sm font-medium text-gray-200">
                  {asset.name}
                </span>
              </div>

              {/* Balance */}
              <div className="text-center text-sm text-gray-300">
                {asset.amount}
              </div>

              {/* Price */}
              <div className="text-center text-sm text-gray-300">
                {asset.price}
              </div>

              {/* Value */}
              <div className="text-center text-sm text-gray-300">
                {asset.value}
              </div>

              {/* Action Buttons */}
              <div className="flex items-center justify-center gap-2.5">
                <button className="px-5 py-1.5 rounded-full border border-gray-600 text-gray-300 text-xs font-medium transition-colors hover:border-gray-500 hover:bg-gray-800/50">
                  Swap
                </button>
                <button className="px-4 py-1.5 rounded-full bg-cyan-500 text-gray-900 text-xs font-semibold whitespace-nowrap transition-colors hover:bg-cyan-400">
                  ADD TO PLAN
                </button>
              </div>

              {/* External Link Icon */}
              <div className="flex justify-center">
                <ArrowTopRightOnSquareIcon className="w-4 h-4 text-gray-400 cursor-pointer hover:text-gray-300 transition-colors" />
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

export default Portfolio;
