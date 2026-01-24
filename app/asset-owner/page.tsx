import {
  ArrowUpRight,
  ChevronDown,
  Clock,
  HandCoins,
  Plus,
  SlidersHorizontal,
  UserPlus,
} from "lucide-react";
import React from "react";
import ActivityBox from "./components/ActivityBox";

function page() {
  return (
    <div>
      {/* Header */}
      <div className="flex justify-between items-start mb-6">
        <div>
          <h1 className="mb-1 text-2xl font-medium text-[#FCFFFF]">
            Good morning, EBUBE
          </h1>
          <p className="text-sm/[24px] text-[#92A5A8]">
            Monitor, protect, and manage the platform.
          </p>
        </div>
        {/* Mobile History Icon */}
        <div className="flex flex-col items-center gap-y-1 md:hidden">
          <span className="text-[10px] text-[#92A5A8]">History</span>
          <button className="text-[#92A5A8]">
            <Clock size={20} />
          </button>
        </div>
      </div>

      {/* Mobile Stats Grid (2 cols, simple cards) */}
      <div className="rounded-b-[48px] overflow-hidden grid grid-cols-2 gap-5 mb-8 md:hidden">
        <div className="px-[22px] py-8 bg-[#182024] text-center flex flex-col items-center rounded-3xl">
          <h3 className="text-4xl/10 text-[#FCFFFF] font-semibold">0</h3>
          <p className="mt-2 text-sm/4 text-[#92A5A8]">Active Plans</p>
        </div>
        <div className="px-[22px] py-8 bg-[#182024] text-center flex flex-col items-center rounded-3xl">
          <h3 className="text-4xl/10 text-[#FCFFFF] font-semibold">0</h3>
          <p className="mt-2 text-sm/4 text-[#92A5A8]">Guardians</p>
        </div>
        <div className="px-[22px] py-8 bg-[#182024] text-center flex flex-col items-center rounded-3xl">
          <h3 className="text-4xl/10 text-[#FCFFFF] font-semibold">0</h3>
          <p className="mt-2 text-sm/4 text-[#92A5A8]">Created Plans</p>
        </div>
        <div className="px-[22px] py-8 bg-[#182024] text-center flex flex-col items-center rounded-3xl">
          <h3 className="text-4xl/10 text-[#FCFFFF] font-semibold">0</h3>
          <p className="mt-2 text-sm/4 text-[#92A5A8]">Pending Claims</p>
        </div>
      </div>

      {/* Desktop Stats Grid (4 cols, cards with buttons) */}
      <div className="rounded-b-[48px] overflow-hidden hidden md:grid grid-cols-4 gap-x-5 mb-16">
        <div className="px-[22px] py-8 bg-[#182024] text-center flex flex-col items-center rounded-xl">
          <h3 className="text-4xl/10 text-[#FCFFFF] font-semibold">0</h3>
          <p className="mb-6 mt-2 text-sm/4 text-[#92A5A8]">Active Plans</p>
          <button className="bg-[#33C5E014] border border-[#33C5E03D] w-[208px] py-4 px-6 flex justify-center items-center gap-x-1 rounded-full text-[#33C5E0]">
            <ArrowUpRight /> Create Plan
          </button>
        </div>
        <div className="px-[22px] py-8 bg-[#182024] text-center flex flex-col items-center rounded-xl">
          <h3 className="text-4xl/10 text-[#FCFFFF] font-semibold">0</h3>
          <p className="mb-6 mt-2 text-sm/4 text-[#92A5A8]">To Withdraw</p>
          <button className="bg-[#33C5E014] border border-[#33C5E03D] w-[208px] py-4 px-6 flex justify-center items-center gap-x-1 rounded-full text-[#33C5E0]">
            <ArrowUpRight /> Withdraw Asset
          </button>
        </div>
        <div className="px-[22px] py-8 bg-[#182024] text-center flex flex-col items-center rounded-xl">
          <h3 className="text-4xl/10 text-[#FCFFFF] font-semibold">0</h3>
          <p className="mb-6 mt-2 text-sm/4 text-[#92A5A8]">created plans</p>
          <button className="bg-[#33C5E014] border border-[#33C5E03D] w-[208px] py-4 px-6 flex justify-center items-center gap-x-1 rounded-full text-[#33C5E0]">
            <ArrowUpRight /> Add Beneficiary
          </button>
        </div>
        <div className="px-[22px] py-8 bg-[#182024] text-center flex flex-col items-center rounded-xl">
          <h3 className="text-4xl/10 text-[#FCFFFF] font-semibold">0</h3>
          <p className="mb-6 mt-2 text-sm/4 text-[#92A5A8]">Pending claims</p>
          <button className="bg-[#33C5E014] border border-[#33C5E03D] w-[208px] py-4 px-6 flex justify-center items-center gap-x-1 rounded-full text-[#33C5E0]">
            <ArrowUpRight /> View claims
          </button>
        </div>
      </div>

      {/* Mobile Actions Row */}
      <div className="flex gap-4 overflow-x-auto pb-4 mb-8 md:hidden">
        <div className="flex flex-col items-center gap-y-2 min-w-[100px]">
          <button className="w-full aspect-[2/1] bg-[#33C5E0] rounded-[24px] flex justify-center items-center text-[#161E22]">
            <Plus size={24} />
          </button>
          <span className="text-xs text-[#33C5E0] font-medium">
            Create Plan
          </span>
        </div>
        <div className="flex flex-col items-center gap-y-2 min-w-[100px]">
          <button className="w-full aspect-[2/1] border border-[#2A3338] rounded-[24px] flex justify-center items-center text-[#33C5E0]">
            <HandCoins size={24} />
          </button>
          <span className="text-xs text-[#33C5E0] font-medium">
            Withdraw Assets
          </span>
        </div>
        <div className="flex flex-col items-center gap-y-2 min-w-[100px]">
          <button className="w-full aspect-[2/1] border border-[#2A3338] rounded-[24px] flex justify-center items-center text-[#33C5E0]">
            <UserPlus size={24} />
          </button>
          <span className="text-xs text-[#33C5E0] font-medium">
            Add Beneficiary
          </span>
        </div>
      </div>

      {/* Mobile Recent Activities (Empty State) */}
      <div className="bg-[#182024]/50 rounded-[32px] p-6 min-h-[400px] md:hidden">
        <div className="flex justify-between items-center mb-6">
          <h2 className="text-sm font-medium text-[#FCFFFF] uppercase tracking-wide">
            RECENT ACTIVITIES
          </h2>
          <button className="flex items-center gap-x-2 text-[#92A5A8] bg-[#1C252A] px-3 py-1.5 rounded-lg text-xs">
            All
            <ChevronDown size={14} />
          </button>
        </div>

        <div className="flex flex-col items-center justify-center h-[250px] text-center">
          <h3 className="text-lg text-[#FCFFFF] mb-1">No activity yet.</h3>
          <p className="text-sm text-[#92A5A8] max-w-[280px] mb-6">
            Add Beneficiaries, Add Guardians or Create Plans to get started
          </p>
          <button className="bg-[#33C5E0] text-[#161E22] py-3 px-6 rounded-full font-medium flex items-center gap-x-2">
            <Plus size={20} />
            Create New Plan
          </button>
        </div>
      </div>

      {/* Desktop Recent Activities (Original List) */}
      <div className="hidden md:block">
        <div className="flex justify-between">
          <div className="w-full">
            <h1 className="py-3 border-b-[#1C252A] border-b-[1px] w-[70%] uppercase font-medium">
              Recent Activities
            </h1>
          </div>
          <button className="flex gap-x-2 items-center text-[#92A5A8]">
            <SlidersHorizontal />
            Filter
          </button>
        </div>
        <div className="flex capitalize mb-3">
          <button className="py-3 px-4">All</button>
          <button className="py-3 px-4">Created Plans</button>
          <button className="py-3 px-4">Swaps</button>
          <button className="py-3 px-4">Inactivity Alert</button>
          <button className="py-3 px-4">Guardians</button>
        </div>

        <ActivityBox />
      </div>
    </div>
  );
}

export default page;
