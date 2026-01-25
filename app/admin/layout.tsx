import { Sidebar } from "@/components/admin/Sidebar";
import { Topbar } from "@/components/admin/Topbar";

export default function AdminLayout({
    children,
}: {
    children: React.ReactNode;
}) {
    return (
        <div className="flex min-h-screen bg-[#060B0D] text-white font-sans">
            {/* Sidebar */}
            <Sidebar />

            {/* Main Content */}
            <div className="flex-1 flex flex-col min-h-screen">
                <Topbar />
                <main className="flex-1 p-6 md:p-10 overflow-auto">
                    {children}
                </main>
            </div>
        </div>
    );
}
