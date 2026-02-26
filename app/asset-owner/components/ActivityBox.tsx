import { div } from "framer-motion/client";
import { Plus } from "lucide-react";
import React from "react";

function ActivityBox() {
  const activities = [];
  return (
    <div className="rounded-[24px] bg-[#182024] min-h-[376px]">
      {activities.length === 0 && (
        <div className="w-full h-full flex flex-col justify-center py-27.5 text-center">
          <h2 className="text-lg/[34px] text-[#FCFFFF]">No activity yet.</h2>
          <p className="text-[#99A9A2] text-xs/[22px] mb-8">
            Add Beneficiaries, Add Guardians or Create Plans to get started
          </p>

          <button className="flex items-center gap-x-2 text-[#161E22] rounded-full bg-[#33C5E0] border border-[#33C5E03D] py-3.5 px-5 w-fit mx-auto">
            <Plus /> Create New Plan
          </button>
        </div>
      )}
    </div>
  );
}

export default ActivityBox;
