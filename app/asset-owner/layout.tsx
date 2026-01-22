import React from "react";
import DashboardNavbar from "../components/layout.tsx/DashboardNavbar";
import Sidebar from "./components/Sidebar";
import MobileBottomNav from "./components/MobileBottomNav";

function layout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <div className="flex flex-col h-[100vh] overflow-hidden bg-[#161E22]">
      <DashboardNavbar />
      <div className="flex-1 flex overflow-hidden flex-col md:flex-row">
        <div className="hidden md:block">
          <Sidebar />
        </div>
        
        <MobileBottomNav />
        
        <div className="overflow-auto flex-1 py-10 px-6 md:pr-12 pb-24 md:pb-10">
          {children}
        </div>
      </div>
    </div>
  );
}

export default layout;
