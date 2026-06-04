"use client";

import { useAppStore } from "@/store/index";
import { locales, localeNames, type Locale } from "@/i18n/config";
import { useRouter, usePathname } from "next/navigation";

export function LanguageSwitcher() {
  const locale = useAppStore((s) => s.locale);
  const setLocale = useAppStore((s) => s.setLocale);
  const router = useRouter();
  const pathname = usePathname();

  const handleChange = (newLocale: Locale) => {
    setLocale(newLocale);
    // next-intl locale prefix routing: replace current locale segment
    const segments = pathname.split("/");
    if (locales.includes(segments[1] as Locale)) {
      segments[1] = newLocale;
    } else {
      segments.splice(1, 0, newLocale);
    }
    router.push(segments.join("/") || "/");
  };

  return (
    <select
      value={locale}
      onChange={(e) => handleChange(e.target.value as Locale)}
      className="bg-[#1C252A] text-[#92A5A8] text-sm rounded-lg px-3 py-1.5 border border-[#2A3338] focus:outline-none focus:border-[#33C5E0]"
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
