import { generateSEO } from "@/lib/seo";

export const metadata = generateSEO({
  title: "Portfolio",
  description:
    "View and manage your digital asset portfolio. Track your crypto holdings and NFTs in one secure dashboard.",
  url: "/asset-owner/portfolio",
  keywords: [
    "crypto portfolio",
    "digital assets",
    "nft management",
    "asset tracking",
    "portfolio dashboard",
  ],
  noindex: true,
});

export default function PortfolioLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return <>{children}</>;
}
