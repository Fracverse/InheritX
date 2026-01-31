import type { Metadata, Viewport } from "next";
import { Outfit, Geist_Mono } from "next/font/google";
import "./globals.css";
import { WalletProvider } from "@/context/WalletContext";
import { AdminAuthProvider } from "@/context/AdminAuthContext";
import { KYCProvider } from "@/context/KYCContext";
import { WalletModal } from "@/components/WalletModal";
import { KYCVerificationModal } from "@/components/KYCVerificationModal";

const outfit = Outfit({
  variable: "--font-outfit",
  subsets: ["latin"],
  display: "swap",
  preload: true,
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
  display: "swap",
});

export const metadata: Metadata = {
  title: "InheritX - Secure Wealth Inheritance & Asset Planning",
  description:
    "InheritX helps you plan and share your assets securely with loved ones. Simple inheritance planning, custom beneficiary rules, and stress-free transfers.",
  keywords: [
    "wealth inheritance",
    "asset planning",
    "digital inheritance",
    "estate planning",
    "legacy planning",
    "beneficiary management",
    "secure transfers",
  ],
  authors: [{ name: "InheritX Team" }],
  creator: "InheritX",
  publisher: "InheritX",
  robots: "index, follow",
  openGraph: {
    type: "website",
    locale: "en_US",
    url: "https://inheritx.com",
    siteName: "InheritX",
    title: "InheritX - Secure Wealth Inheritance & Asset Planning",
    description:
      "Plan your legacy with InheritX. Secure, simple, and stress-free inheritance planning.",
    images: [
      {
        url: "https://inheritx.com/og-image.jpg",
        width: 1200,
        height: 630,
        alt: "InheritX - Secure Wealth Inheritance",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: "InheritX - Secure Wealth Inheritance & Asset Planning",
    description: "Plan your legacy securely with InheritX.",
    images: ["https://inheritx.com/twitter-image.jpg"],
  },
  alternates: {
    canonical: "https://inheritx.com",
  },
};

export const viewport: Viewport = {
  width: "device-width",
  initialScale: 1,
  maximumScale: 5,
  userScalable: true,
  themeColor: "#161E22",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  // We need to import these dynamically or ensure "use client" wraps them if passing context
  // But Layout is a Server Component by default in Next.js App Router.
  // We cannot use Context directly in a Server Component layout.
  // So we need a "Providers" client component or assume WalletProvider is "use client".
  // WalletProvider IS "use client".

  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <meta name="color-scheme" content="dark" />
        <meta name="apple-mobile-web-app-capable" content="yes" />
        <meta
          name="apple-mobile-web-app-status-bar-style"
          content="black-translucent"
        />
        <link rel="icon" href="/favicon.ico" />
        <link rel="apple-touch-icon" href="/apple-touch-icon.png" />
        <link rel="manifest" href="/site.webmanifest" />
      </head>
      <body
        className={`${outfit.variable} ${geistMono.variable} antialiased`}
        suppressHydrationWarning
      >
        <AdminAuthProvider>
          <WalletProvider>
            <KYCProvider>
              <main className="">{children}</main>
              <WalletModal />
              <KYCVerificationModal />
            </KYCProvider>
          </WalletProvider>
        </AdminAuthProvider>
      </body>
    </html>
  );
}
