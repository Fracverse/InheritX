/**
 * Format a Stellar address for display
 * @param address - Full Stellar address
 * @returns Formatted address (e.g., "GABC...WXYZ")
 */
export function formatAddress(address: string): string {
  if (!address || address.length < 8) return address;
  return `${address.slice(0, 4)}...${address.slice(-4)}`;
}
