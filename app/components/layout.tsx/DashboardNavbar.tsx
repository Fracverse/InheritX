"use client";

import { ConnectButton } from "@/components/ConnectButton";
import { useWallet } from "@/context/WalletContext";
import { useKYC } from "@/context/KYCContext";
import useUpdateEffect from "@/hooks/useUpdateEffect";
import {
  EllipsisVertical,
  Search,
  ShieldAlert,
  ShieldCheck,
  CheckCircle,
  XCircle,
} from "lucide-react";
import Image from "next/image";
import { useRouter } from "next/navigation";

function DashboardNavbar() {
  const { isConnected, address } = useWallet();
  const { kycStatus, openKYCModal } = useKYC();
  const router = useRouter();

  useUpdateEffect(() => {
    if (!isConnected && !address) {
      router.push("/");
    }
  }, [isConnected, address, router]);

  const getKYCStatusConfig = () => {
    switch (kycStatus) {
      case "submitted":
        return {
          icon: ShieldCheck,
          title: "KYC Submitted",
          subtitle: "Under review",
          bgColor: "bg-[#ECC94B14]",
          textColor: "text-[#ECC94B]",
          iconColor: "text-[#ECC94B]",
        };
      case "approved":
        return {
          icon: CheckCircle,
          title: "KYC Verified",
          subtitle: "Verified",
          bgColor: "bg-[#48BB7814]",
          textColor: "text-[#48BB78]",
          iconColor: "text-[#48BB78]",
        };
      case "rejected":
        return {
          icon: XCircle,
          title: "KYC Rejected",
          subtitle: "Action required",
          bgColor: "bg-[#F5656514]",
          textColor: "text-[#F56565]",
          iconColor: "text-[#F56565]",
        };
      default:
        return {
          icon: ShieldAlert,
          title: "KYC Verification",
          subtitle: "Action required",
          bgColor: "bg-[#33C5E014]",
          textColor: "text-[#33C5E0]",
          iconColor: "text-[#33C5E0]",
        };
    }
  };

  const statusConfig = getKYCStatusConfig();
  const StatusIcon = statusConfig.icon;

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
          <button
            onClick={openKYCModal}
            className={`${statusConfig.bgColor} rounded-[24px] py-[10px] px-4 flex gap-x-[6px] items-center hover:opacity-80 transition-opacity cursor-pointer`}
          >
            <StatusIcon size={40} className={statusConfig.iconColor} />
            <div>
              <h4 className="text-xs/[16px] font-medium">
                {statusConfig.title}
              </h4>
              <h6 className={`text-[10px] ${statusConfig.textColor}`}>
                {statusConfig.subtitle}
              </h6>
            </div>
          </button>

          <div className="flex gap-x-2">
            <ConnectButton targetUI="dashboard" />

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
