/**
 * API Filtering and Sorting Utilities
 * Provides query parameter parsing and data filtering/sorting capabilities
 */

export interface FilterOptions {
  [key: string]: string | string[] | number | boolean | undefined;
}

export interface SortOptions {
  field: string;
  order: "asc" | "desc";
}

export interface PaginationOptions {
  page: number;
  limit: number;
}

export interface QueryParams {
  filters?: FilterOptions;
  sort?: SortOptions[];
  search?: string;
  pagination?: PaginationOptions;
}

/**
 * Parse URL search params into structured query parameters
 */
export function parseQueryParams(searchParams: URLSearchParams): QueryParams {
  const params: QueryParams = {
    filters: {},
    sort: [],
    pagination: {
      page: 1,
      limit: 10,
    },
  };

  // Parse filters (e.g., ?status=active&type=standard)
  for (const [key, value] of searchParams.entries()) {
    if (key === "sort" || key === "search" || key === "page" || key === "limit") {
      continue;
    }
    
    // Handle array values (e.g., ?status=active&status=pending)
    if (params.filters![key]) {
      const existing = params.filters![key];
      params.filters![key] = Array.isArray(existing) 
        ? [...existing, value] 
        : [existing as string, value];
    } else {
      params.filters![key] = value;
    }
  }

  // Parse sort (e.g., ?sort=created_at:desc,name:asc)
  const sortParam = searchParams.get("sort");
  if (sortParam) {
    params.sort = sortParam.split(",").map((s) => {
      const [field, order = "asc"] = s.split(":");
      return { field, order: order as "asc" | "desc" };
    });
  }

  // Parse search (e.g., ?search=john)
  const searchParam = searchParams.get("search");
  if (searchParam) {
    params.search = searchParam;
  }

  // Parse pagination
  const page = searchParams.get("page");
  const limit = searchParams.get("limit");
  if (page) params.pagination!.page = parseInt(page, 10);
  if (limit) params.pagination!.limit = parseInt(limit, 10);

  return params;
}

/**
 * Apply filters to an array of data
 */
export function applyFilters<T extends Record<string, any>>(
  data: T[],
  filters: FilterOptions
): T[] {
  if (!filters || Object.keys(filters).length === 0) {
    return data;
  }

  return data.filter((item) => {
    return Object.entries(filters).every(([key, value]) => {
      if (value === undefined) return true;

      const itemValue = item[key];

      // Handle array filters (OR logic)
      if (Array.isArray(value)) {
        return value.some((v) => matchValue(itemValue, v));
      }

      return matchValue(itemValue, value);
    });
  });
}

/**
 * Match a value against a filter value
 */
function matchValue(itemValue: any, filterValue: any): boolean {
  if (itemValue === undefined || itemValue === null) return false;

  // Exact match for booleans and numbers
  if (typeof filterValue === "boolean" || typeof filterValue === "number") {
    return itemValue === filterValue;
  }

  // String comparison (case-insensitive)
  const itemStr = String(itemValue).toLowerCase();
  const filterStr = String(filterValue).toLowerCase();

  // Support wildcards (* and ?)
  if (filterStr.includes("*") || filterStr.includes("?")) {
    const regex = new RegExp(
      "^" + filterStr.replace(/\*/g, ".*").replace(/\?/g, ".") + "$"
    );
    return regex.test(itemStr);
  }

  return itemStr === filterStr;
}

/**
 * Apply search to an array of data
 * Searches across all string fields
 */
export function applySearch<T extends Record<string, any>>(
  data: T[],
  search: string,
  searchFields?: string[]
): T[] {
  if (!search) return data;

  const searchLower = search.toLowerCase();

  return data.filter((item) => {
    const fieldsToSearch = searchFields || Object.keys(item);

    return fieldsToSearch.some((field) => {
      const value = item[field];
      if (value === undefined || value === null) return false;

      return String(value).toLowerCase().includes(searchLower);
    });
  });
}

/**
 * Apply sorting to an array of data
 */
export function applySort<T extends Record<string, any>>(
  data: T[],
  sortOptions: SortOptions[]
): T[] {
  if (!sortOptions || sortOptions.length === 0) {
    return data;
  }

  return [...data].sort((a, b) => {
    for (const { field, order } of sortOptions) {
      const aValue = a[field];
      const bValue = b[field];

      // Handle null/undefined
      if (aValue === null || aValue === undefined) return order === "asc" ? 1 : -1;
      if (bValue === null || bValue === undefined) return order === "asc" ? -1 : 1;

      // Compare values
      let comparison = 0;
      if (typeof aValue === "string" && typeof bValue === "string") {
        comparison = aValue.localeCompare(bValue);
      } else if (typeof aValue === "number" && typeof bValue === "number") {
        comparison = aValue - bValue;
      } else if (aValue instanceof Date && bValue instanceof Date) {
        comparison = aValue.getTime() - bValue.getTime();
      } else {
        comparison = String(aValue).localeCompare(String(bValue));
      }

      if (comparison !== 0) {
        return order === "asc" ? comparison : -comparison;
      }
    }

    return 0;
  });
}

/**
 * Apply pagination to an array of data
 */
export function applyPagination<T>(
  data: T[],
  pagination: PaginationOptions
): { data: T[]; total: number; page: number; limit: number; totalPages: number } {
  const { page, limit } = pagination;
  const total = data.length;
  const totalPages = Math.ceil(total / limit);
  const start = (page - 1) * limit;
  const end = start + limit;

  return {
    data: data.slice(start, end),
    total,
    page,
    limit,
    totalPages,
  };
}

/**
 * Apply all query operations (filter, search, sort, paginate)
 */
export function applyQueryParams<T extends Record<string, any>>(
  data: T[],
  params: QueryParams,
  searchFields?: string[]
): {
  data: T[];
  total: number;
  page: number;
  limit: number;
  totalPages: number;
  filters: FilterOptions;
  sort: SortOptions[];
  search?: string;
} {
  let result = [...data];

  // Apply filters
  if (params.filters) {
    result = applyFilters(result, params.filters);
  }

  // Apply search
  if (params.search) {
    result = applySearch(result, params.search, searchFields);
  }

  // Apply sort
  if (params.sort && params.sort.length > 0) {
    result = applySort(result, params.sort);
  }

  // Apply pagination
  const paginationResult = applyPagination(
    result,
    params.pagination || { page: 1, limit: 10 }
  );

  return {
    ...paginationResult,
    filters: params.filters || {},
    sort: params.sort || [],
    search: params.search,
  };
}

/**
 * Build query string from query params
 */
export function buildQueryString(params: QueryParams): string {
  const searchParams = new URLSearchParams();

  // Add filters
  if (params.filters) {
    Object.entries(params.filters).forEach(([key, value]) => {
      if (Array.isArray(value)) {
        value.forEach((v) => searchParams.append(key, String(v)));
      } else if (value !== undefined) {
        searchParams.set(key, String(value));
      }
    });
  }

  // Add sort
  if (params.sort && params.sort.length > 0) {
    const sortStr = params.sort.map((s) => `${s.field}:${s.order}`).join(",");
    searchParams.set("sort", sortStr);
  }

  // Add search
  if (params.search) {
    searchParams.set("search", params.search);
  }

  // Add pagination
  if (params.pagination) {
    searchParams.set("page", String(params.pagination.page));
    searchParams.set("limit", String(params.pagination.limit));
  }

  return searchParams.toString();
}
