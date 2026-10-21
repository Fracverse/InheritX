/**
 * CSV Export Utilities for Admin Reports
 */

export interface ExportOptions {
  filename?: string;
  headers?: string[];
  delimiter?: string;
}

/**
 * Convert array of objects to CSV string
 */
export function toCsvString<T extends Record<string, any>>(
  data: T[],
  options: ExportOptions = {}
): string {
  if (!data || data.length === 0) {
    return '';
  }

  const delimiter = options.delimiter || ',';
  const headers = options.headers || Object.keys(data[0]);

  // Escape CSV values
  const escapeValue = (value: any): string => {
    if (value === null || value === undefined) {
      return '';
    }

    const stringValue = String(value);
    
    // Check if value needs to be quoted
    if (
      stringValue.includes(delimiter) ||
      stringValue.includes('\n') ||
      stringValue.includes('\r') ||
      stringValue.includes('"')
    ) {
      // Escape quotes by doubling them
      return `"${stringValue.replace(/"/g, '""')}"`;
    }

    return stringValue;
  };

  // Build CSV lines
  const lines: string[] = [];

  // Add header row
  lines.push(headers.map(h => escapeValue(h)).join(delimiter));

  // Add data rows
  for (const row of data) {
    const values = headers.map(header => {
      const value = row[header];
      return escapeValue(value);
    });
    lines.push(values.join(delimiter));
  }

  return lines.join('\n');
}

/**
 * Download CSV file
 */
export function downloadCsv<T extends Record<string, any>>(
  data: T[],
  options: ExportOptions = {}
): void {
  const csv = toCsvString(data, options);
  const filename = options.filename || 'export.csv';

  // Create blob
  const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });

  // Create download link
  const link = document.createElement('a');
  const url = URL.createObjectURL(blob);

  link.setAttribute('href', url);
  link.setAttribute('download', filename);
  link.style.visibility = 'hidden';

  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);

  // Clean up
  URL.revokeObjectURL(url);
}

/**
 * Export payouts table as CSV
 */
export function exportPayoutsAsCsv(payouts: any[], filename = 'payouts.csv'): void {
  const headers = ['Payout ID', 'Plan ID', 'Beneficiary', 'Amount', 'Currency', 'Anchor Partner', 'Status', 'Settlement Date'];
  
  const data = payouts.map(payout => ({
    'Payout ID': payout.id,
    'Plan ID': payout.planId,
    'Beneficiary': payout.beneficiary,
    'Amount': payout.amount,
    'Currency': payout.currency,
    'Anchor Partner': payout.anchor,
    'Status': payout.status,
    'Settlement Date': new Date(payout.date).toLocaleDateString(),
  }));

  downloadCsv(data, { filename, headers });
}

/**
 * Export users table as CSV
 */
export function exportUsersAsCsv(users: any[], filename = 'users.csv'): void {
  const headers = ['Wallet Address', 'KYC Status', 'Active Plans', 'Status', 'Join Date'];
  
  const data = users.map(user => ({
    'Wallet Address': user.walletAddress,
    'KYC Status': user.kycStatus,
    'Active Plans': user.activePlansCount,
    'Status': user.status,
    'Join Date': new Date(user.joinDate).toLocaleDateString(),
  }));

  downloadCsv(data, { filename, headers });
}
