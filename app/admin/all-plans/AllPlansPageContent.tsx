"use client"

import { useState } from 'react';
import { Search, AlertTriangle } from 'lucide-react';

export default function AllPlansPageContent() {
    const [activeTab, setActiveTab] = useState('All');
    const [searchQuery, setSearchQuery] = useState('');

    // Simulated wallet connection status
    const [isWalletConnected, setIsWalletConnected] = useState(false);

    const tabs = ['All', 'Active', 'Paused', 'Executed', 'Cancelled'];

    const allPlans = [
        {
            id: 1,
            name: 'Testnet testing',
            description: 'Just testing out inheritx dapp',
            owner: '–',
            amount: '0.000000001 USDC',
            beneficiaries: 1,
            transferDate: 'Jan 19, 2026',
            status: 'PENDING'
        },
        {
            id: 2,
            name: 'Testing something',
            description: 'Hello soemthing nnew',
            owner: '–',
            amount: '0.0001 ETH',
            beneficiaries: 2,
            transferDate: 'Oct 10, 2928',
            status: 'ACTIVE'
        },
        {
            id: 3,
            name: 'Wedding Fund',
            description: 'Fund my daughters plan',
            owner: '–',
            amount: '0.0001 ETH',
            beneficiaries: 2,
            transferDate: 'Jan 10, 2027',
            status: 'PAUSED'
        },
        {
            id: 4,
            name: 'Wedding Fund',
            description: 'Fund my daughters plan',
            owner: '–',
            amount: '0.0001 ETH',
            beneficiaries: 2,
            transferDate: 'Jan 10, 2027',
            status: 'EXECUTED'
        },
        {
            id: 5,
            name: 'Wedding Fund',
            description: 'Fund my daughters plan',
            owner: '–',
            amount: '0.0001 ETH',
            beneficiaries: 2,
            transferDate: 'Jan 10, 2027',
            status: 'CANCELLED'
        }
    ];

    // Filter plans based on active tab and search query
    const filteredPlans = allPlans.filter(plan => {
        // Filter by tab
        const matchesTab = activeTab === 'All' || plan.status === activeTab.toUpperCase();

        // Filter by search query
        const matchesSearch = searchQuery === '' ||
            plan.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
            plan.description.toLowerCase().includes(searchQuery.toLowerCase());

        return matchesTab && matchesSearch;
    });

    return (
        <div className="max-w-[98vw] md:max-w-auto text-white px-2 -mt-5 sm:-mx-5">
            <div className="mx-auto">
                {/* Warning Banner */}
                {!isWalletConnected &&
                    <div className="mb-8 bg-linear-to-r from-yellow-900/40 via-yellow-800/30 to-yellow-900/40 border border-yellow-700/50 rounded-xl p-3 md:p-2.5 flex items-center md:items-center flex-wrap justify-between gap-4 backdrop-blur-sm hover:border-yellow-600/60 transition-all duration-3000 animate-slide-up" style={{ animationDelay: '0.3s' }}>
                        <div className="flex items-center gap-3 md:gap-4 flex-1">
                            <AlertTriangle className="w-5 h-5 md:w-6 md:h-6 text-yellow-500 shrink-0 " />
                            <div className="flex-1">
                                <h3 className="text-yellow-500 font-semibold text-sm md:text-base">
                                    Wallet not connected
                                </h3>
                                <p className="text-gray-400 text-xs md:text-sm">
                                    Connect your wallet to perform blockchain operations like KYC approval.
                                </p>
                            </div>
                        </div>
                        <button className="bg-[#33c5e0] hover:bg-cyan-300 text-black font-semibold px-4 md:px-6 py-2 md:py-2 rounded-lg text-sm md:text-base transition-all duration-200 hover:scale-105 active:scale-95 hover:shadow-lg hover:shadow-cyan-500/50 whitespace-nowrap shrink-0">
                            Connect Wallet
                        </button>
                    </div>
                }

                {/* Header */}
                <div className="mb-4 animate-slide-up" style={{ animationDelay: '0.3s' }}>
                    <h1 className="text-2xl md:text-3xl font-semibold mb-2">All Plans</h1>
                    <p className="text-gray-400 text-sm md:text-base">
                        View all inheritance plans on the platform.
                    </p>
                </div>

                {/* Search and Filters */}
                <div className="mb-6 flex flex-col md:flex-row gap-4 items-stretch md:items-center justify-between animate-fade-in" style={{ animationDelay: '0.3s' }}>
                    {/* Search Bar */}
                    <div className="relative flex-1 max-w-2xl group">
                        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-500 group-focus-within:text-primary transition-colors duration-200" />
                        <input
                            type="text"
                            placeholder="Search plans..."
                            value={searchQuery}
                            onChange={(e) => setSearchQuery(e.target.value)}
                            className="w-full border border-gray-800 rounded-lg pl-10 pr-4 py-2.5 text-sm focus:outline-none focus:border-cyan-500/50 focus:ring-2 focus:ring-cyan-500/20 transition-all duration-200 hover:border-gray-700"
                        />
                    </div>

                    {/* Tab Filters */}
                    <div className="flex gap-2 overflow-x-auto px-2 scrollbar-hide">
                        {tabs.map((tab, index) => (
                            <button
                                key={tab}
                                onClick={() => setActiveTab(tab)}
                                className={`px-3 my-1.5 md:px-3 py-2 md:py-2.5 rounded-lg font-medium text-xs md:text-sm whitespace-nowrap transition-all duration-300 transform hover:scale-105 ${activeTab === tab
                                    ? 'bg-[#33c5e0] text-black hover:bg-cyan-300 shadow-lg'
                                    : 'bg-gray-900/50 text-gray-100 hover:bg-gray-800 hover:text-white'
                                    }`}
                                style={{ animationDelay: `${0.2 + index * 0.05}s` }}
                            >
                                {tab}
                            </button>
                        ))}
                    </div>
                </div>

                {/* Table */}
                <div className=" rounded-xl mb-10 sm:mb-0 border border-gray-800 animate-slide-up" style={{ animationDelay: '0.3s' }}>
                    {/* Container with horizontal scroll indicator - mobile only */}
                    <div className="relative">
                        {/* Fade overlay for scroll indication */}
                        <div className="md:hidden absolute inset-y-0 right-0 w-8 bg-linear-to-l from-gray-900/30 to-transparent z-10 pointer-events-none"></div>

                        {/* Scrollable container - fixed for mobile scrolling */}
                        <div className="overflow-auto scrollbar-thin scrollbar-thumb-gray-700 scrollbar-track-transparent pb-2 md:pb-0 [-webkit-overflow-scrolling:touch] [touch-action:pan-x]">
                            <table className="w-full min-h-full min-w-200 md:min-w-full table-auto">
                                {/* Table Header */}
                                <thead>
                                    <tr className="bg-slate-800/60 border-b border-gray-800">
                                        <th className="px-4 py-4 md:px-6 md:py-4 text-left text-xs uppercase tracking-wider text-gray-500 font-semibold whitespace-nowrap">
                                            Plan
                                        </th>
                                        <th className="px-4 py-4 md:px-6 md:py-4 text-left text-xs uppercase tracking-wider text-gray-500 font-semibold whitespace-nowrap">
                                            Owner
                                        </th>
                                        <th className="px-4 py-4 md:px-6 md:py-4 text-left text-xs uppercase tracking-wider text-gray-500 font-semibold whitespace-nowrap">
                                            Amount
                                        </th>
                                        <th className="px-4 py-4 md:px-6 md:py-4 text-left text-xs uppercase tracking-wider text-gray-500 font-semibold whitespace-nowrap">
                                            Beneficiaries
                                        </th>
                                        <th className="px-4 py-4 md:px-6 md:py-4 text-left text-xs uppercase tracking-wider text-gray-500 font-semibold whitespace-nowrap">
                                            Transfer Date
                                        </th>
                                        <th className="px-4 py-4 md:px-6 md:py-4 text-left text-xs uppercase tracking-wider text-gray-500 font-semibold whitespace-nowrap">
                                            Status
                                        </th>
                                    </tr>
                                </thead>

                                {/* Table Body */}
                                <tbody className="divide-y divide-gray-800">
                                    {filteredPlans.length > 0 ? (
                                        filteredPlans.map((plan) => (
                                            <tr
                                                key={plan.id}
                                                className="hover:bg-gray-900/40 transition-all duration-200 cursor-pointer group"

                                            >
                                                {/* Plan */}
                                                <td className="px-4 py-3 md:px-6 md:py-3.5">
                                                    <h3 className="font-semibold text-white group-hover:text-cyan-400 transition-colors duration-200 whitespace-nowrap">
                                                        {plan.name}
                                                    </h3>
                                                    <p className="text-sm text-gray-500 mt-0.5 max-w-50 md:max-w-82.5 lg:max-w-none truncate">
                                                        {plan.description}
                                                    </p>
                                                </td>

                                                {/* Owner */}
                                                <td className="px-4 py-3 md:px-6 md:py-3.5">
                                                    <div className="text-gray-400 whitespace-nowrap">{plan.owner}</div>
                                                </td>

                                                {/* Amount */}
                                                <td className="px-4 py-3 md:px-6 md:py-3.5">
                                                    <div className="font-mono text-sm text-white whitespace-nowrap">{plan.amount}</div>
                                                </td>

                                                {/* Beneficiaries */}
                                                <td className="px-4 py-3 md:px-6 md:py-3.5">
                                                    <div className="text-white whitespace-nowrap">{plan.beneficiaries}</div>
                                                </td>

                                                {/* Transfer Date */}
                                                <td className="px-4 py-3 md:px-6 md:py-3.5">
                                                    <div className="text-white whitespace-nowrap">{plan.transferDate}</div>
                                                </td>

                                                {/* Status */}
                                                <td className="px-4 py-3 md:px-6 md:py-3.5">
                                                    <span className="inline-block px-3 py-1 rounded-md text-xs font-semibold bg-purple-500/20 text-purple-400 border border-purple-500/30 transition-all duration-200 group-hover:bg-purple-500/30 group-hover:border-purple-500/50 whitespace-nowrap">
                                                        {plan.status}
                                                    </span>
                                                </td>
                                            </tr>
                                        ))
                                    ) : (
                                        <tr>
                                            <td colSpan={6} className="px-4 py-8 md:px-6 md:py-12 text-center animate-fade-in">
                                                <div className="flex flex-col items-center justify-center">
                                                    <p className="text-gray-500 text-base md:text-lg">No plans found</p>
                                                    <p className="text-gray-600 text-sm mt-2 max-w-75">
                                                        {searchQuery ? 'Try adjusting your search query' : 'Change your filter to see more plans'}
                                                    </p>
                                                </div>
                                            </td>
                                        </tr>
                                    )}
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}