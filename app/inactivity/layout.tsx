import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Account Inactivity - InheritX",
  description: "Manage your InheritX account inactivity settings",
};

export default function InactivityLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return <>{children}</>;
}
