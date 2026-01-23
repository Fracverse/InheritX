import React from "react";
import { ChevronDown } from "lucide-react";

function page() {
  const stats = [
    { value: '0', label: 'Total Portfolio Value', link: 'See Asset Cycle Tree' },
    { value: '0', label: 'Opened Count', link: 'My Asset Count Pie' },
    { value: '0', label: 'NFT count', link: 'My Asset Cycle Tree' },
    { value: '0', label: 'Recent Images', link: 'Test images' }
  ];

  return (
    <div className="min-h-screen text-white p-4 sm:p-6">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-2xl sm:text-3xl font-semibold mb-1">Portfolio</h1>
        <p className="text-gray-400 text-sm sm:text-base">Click on the display of your wealth so far.</p>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-12">
        {stats.map((stat, index) => (
          <div 
            key={index}
            className="rounded-lg p-4 sm:p-6 border border-gray-800 hover:border-gray-700 transition-colors"
          >
            <div className="text-3xl sm:text-4xl font-bold mb-2">{stat.value}</div>
            <div className="text-gray-400 text-sm mb-4">{stat.label}</div>
            <button className="border-1 p-2 border-gray-700 rounded-full text-gray-400 text-sm flex items-center gap-1 hover:gap-2 transition-all">
              {stat.link}
              {stat.link === 'Test images' ? <ChevronDown size={16} /> : null}
            </button>
          </div>
        ))}
      </div>

      {/* Charts & Insights Section */}
      <div className="mb-9">
        <h2 className="text-xl sm:text-2xl font-semibold mb-6">CHARTS & INSIGHTS</h2>
        
        {/* No Assets Found */}
        <div className="rounded-lg p-6 sm:p-8 md:p-12 border border-gray-800 text-center">
          <div className="mb-2">
            <h3 className="text-lg font-medium mb-1">No assets found.</h3>
            <p className="text-gray-400 text-sm">Connect your wallet to get started.</p>
          </div>
          <button className="mt-6 bg-cyan-500 hover:bg-cyan-600 text-white px-6 py-2 rounded-full text-sm font-medium transition-colors flex items-center justify-center gap-2 mx-auto">
            <span className="text-lg">+</span>
            Swap Assets
          </button>
        </div>
      </div>
    </div>
  )
}

export default page;
