"use client";

import { useLocale } from "next-intl";
import { useRouter, usePathname } from "next/navigation";
import { useTransition } from "react";
import { locales, localeNames, type Locale } from "@/i18n/config";

export function LanguageSwitcher() {
  const locale = useLocale();
  const router = useRouter();
  const pathname = usePathname();
  const [isPending, startTransition] = useTransition();

  const handleChange = (newLocale: Locale) => {
    // next-intl middleware reads NEXT_LOCALE cookie for preference persistence.
    // Setting it here ensures the selection survives hard reloads and new tabs.
    document.cookie = `NEXT_LOCALE=${newLocale}; path=/; max-age=31536000; SameSite=Lax`;

    startTransition(() => {
      // Strip any existing locale prefix then push the new one.
      // With localePrefix "as-needed", the default locale ("en") has no prefix.
      const segments = pathname.split("/").filter(Boolean);
      if (locales.includes(segments[0] as Locale)) {
        segments.shift();
      }
      const newPath =
        newLocale === "en"
          ? "/" + segments.join("/")
          : `/${newLocale}/${segments.join("/")}`;
      router.push(newPath || "/");
    });
  };

  return (
    <select
      value={locale}
      onChange={(e) => handleChange(e.target.value as Locale)}
      disabled={isPending}
      className="bg-[#1C252A] text-[#92A5A8] text-sm rounded-lg px-3 py-1.5 border border-[#2A3338] focus:outline-none focus:border-[#33C5E0] disabled:opacity-60 cursor-pointer"
      aria-label="Select language"
    >
      {locales.map((l) => (
        <option key={l} value={l}>
          {localeNames[l]}
        </option>
      ))}
    </select>
  );
}
