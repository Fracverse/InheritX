import { generateSEO } from "@/lib/seo";

export const metadata = generateSEO({
  title: "My Plans",
  description:
    "Create and manage your inheritance plans. Set up beneficiaries, define asset distribution rules, and secure your legacy.",
  url: "/asset-owner/plans",
  keywords: [
    "inheritance plans",
    "estate planning",
    "beneficiary setup",
    "asset distribution",
    "legacy planning",
  ],
  noindex: true,
});

export default function PlansLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return <>{children}</>;
}
