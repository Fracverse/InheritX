import { EllipsisVertical, Search, ShieldAlert } from "lucide-react";
import Image from "next/image";
import React from "react";

function DashboardNavbar() {
  return (
    <div className="py-4 px-4 md:px-12 flex items-center justify-between border-b-[1px] border-b-[#1C252A]">
      <Image
        src="/logo.svg"
        alt="InheritX"
        width={50}
        height={50}
        quality={85}
      />

      <div className="gap-x-8 flex items-center">
        <div className="relative w-fit h-fit text-[#92A5A8] hidden md:block">
          <Search
            className="absolute top-1/2 -translate-y-1/2 left-5"
            size={18}
          />
          <input
            type="text"
            className="p-4 pl-[48px] rounded-full border-[1px] w-[350px] border-[#2A3338]"
            placeholder="Search user, Ticket ID, Plans, & Admins..."
          />
        </div>

        <div className="flex items-center gap-x-2">
          <div className="bg-[#33C5E014] rounded-[24px] py-[10px] px-4 flex gap-x-[6px] items-center">
            <ShieldAlert size={40} className="text-[#33C5E0]" />
            <div>
              <h4 className="text-xs/[16px] font-medium">KYC Verification</h4>
              <h6 className="text-[10px] text-[#33C5E0]">Action required</h6>
            </div>
          </div>
          <div className="flex gap-x-2">
            <div className="p-4 rounded-l-full flex items-center gap-x-[6px] bg-[#1C252A] hidden md:flex">
              <img src="/user-avatar.svg" alt="" />
              <span className="leading-6 text-[#92A5A8]">0x123Htp....5678</span>
            </div>
            <button className="bg-[#1C252A] p-4 rounded-[8px]">
              <span className="-rotate-18o">
                <EllipsisVertical />
              </span>
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

export default DashboardNavbar;
