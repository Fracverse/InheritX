import { DashboardLayout } from "@/components/dashboard/DashboardLayout";

export default function AssetOwnerLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return <DashboardLayout>{children}</DashboardLayout>;
}
