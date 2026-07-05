export type KYCStatus = "pending" | "approved" | "rejected";
export type UserStatus = "active" | "suspended";

export interface AdminUser {
  id: string;
  walletAddress: string;
  registrationDate: string;
  kycStatus: KYCStatus;
  activePlansCount: number;
  tvl: number;
  status: UserStatus;
}

export interface UserManagementTableProps {
  users: AdminUser[];
  onApproveKYC: (userId: string) => Promise<void>;
  onTerminatePlan: (userId: string) => Promise<void>;
  onSuspendUser: (userId: string) => Promise<void>;
}

export interface SearchFilterBarProps {
  onSearchChange: (value: string) => void;
  onFilterChange: (value: KYCStatus | "all") => void;
}

export interface PaginationProps {
  currentPage: number;
  totalPages: number;
  onPageChange: (page: number) => void;
}
