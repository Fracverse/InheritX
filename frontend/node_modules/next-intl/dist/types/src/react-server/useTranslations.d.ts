import type { useTranslations as useTranslationsType } from 'use-intl';
export default function useTranslations(...[namespace]: Parameters<typeof useTranslationsType>): ReturnType<typeof useTranslationsType>;
