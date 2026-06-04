import React from "react";
import { Plus, ArrowUpRight } from "lucide-react";

const RecentTransactions = () => {
  return (
    <div className="w-full mt-12">
      <h2 className="text-lg font-medium text-[#FCFFFF] mb-4">
        Recent transactions
      </h2>

      <div className="bg-[#182024]/50 rounded-4xl p-8 md:p-24 border border-[#2A3338] flex flex-col items-center justify-center text-center">
        <h3 className="text-xl md:text-2xl font-semibold text-[#FCFFFF] mb-2">
          You don&apos;t have an activity record yet.
        </h3>
        <p className="text-sm md:text-base text-[#92A5A8] mb-8 max-w-md">
          Swap assets and add to a plan to see your transaction history
        </p>

        <button className="bg-[#33C5E0] text-[#161E22] py-4 px-8 rounded-full font-bold flex items-center gap-2 hover:bg-[#2EB0C8] transition-all transform active:scale-95 shadow-[0_0_20px_rgba(51,197,224,0.3)]">
          <ArrowUpRight size={20} />
          Swap Assets
        </button>
      </div>
    </div>
  );
};

export default RecentTransactions;
