/**
 * This is only moved to a separate module for easier mocking in
 * `../createNavigatoin.test.tsx` in order to avoid suspending.
 */
export default function getServerLocale(): Promise<string>;
