import React, { ReactNode } from 'react';
import IntlConfig from '../core/IntlConfig';
type Props = IntlConfig & {
    children: ReactNode;
};
export default function IntlProvider({ children, defaultTranslationValues, formats, getMessageFallback, locale, messages, now, onError, timeZone }: Props): React.JSX.Element;
export {};
