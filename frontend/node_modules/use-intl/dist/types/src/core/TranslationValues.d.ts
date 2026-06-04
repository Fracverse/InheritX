import { ReactNode } from 'react';
export type TranslationValue = string | number | boolean | Date | null | undefined;
type TranslationValues = Record<string, TranslationValue>;
export type RichTranslationValues = Record<string, TranslationValue | ((chunks: ReactNode) => ReactNode)>;
export type MarkupTranslationValues = Record<string, TranslationValue | ((chunks: string) => string)>;
export default TranslationValues;
