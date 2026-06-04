/**
 * React hook for API queries with filtering, sorting, and pagination
 */
import { useState, useEffect, useCallback } from "react";
import { QueryParams, buildQueryString } from "@/lib/api/filtering";

interface UseApiQueryOptions<T> {
  endpoint: string;
  initialParams?: Partial<QueryParams>;
  enabled?: boolean;
  onSuccess?: (data: T[]) => void;
  onError?: (error: Error) => void;
}

interface ApiResponse<T> {
  status: string;
  data: T[];
  pagination?: {
    total: number;
    page: number;
    limit: number;
    totalPages: number;
  };
  filters?: Record<string, any>;
  sort?: Array<{ field: string; order: "asc" | "desc" }>;
}

export function useApiQuery<T = any>(options: UseApiQueryOptions<T>) {
  const { endpoint, initialParams, enabled = true, onSuccess, onError } = options;

  const [data, setData] = useState<T[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [pagination, setPagination] = useState({
    total: 0,
    page: 1,
    limit: 10,
    totalPages: 0,
  });

  const [queryParams, setQueryParams] = useState<QueryParams>({
    filters: initialParams?.filters || {},
    sort: initialParams?.sort || [],
    search: initialParams?.search,
    pagination: initialParams?.pagination || { page: 1, limit: 10 },
  });

  const fetchData = useCallback(async () => {
    if (!enabled) return;

    setLoading(true);
    setError(null);

    try {
      const queryString = buildQueryString(queryParams);
      const url = `${endpoint}?${queryString}`;
      const response = await fetch(url);

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const result: ApiResponse<T> = await response.json();
      
      setData(result.data);
      if (result.pagination) {
        setPagination(result.pagination);
      }

      if (onSuccess) {
        onSuccess(result.data);
      }
    } catch (err) {
      const error = err instanceof Error ? err : new Error("Unknown error");
      setError(error);
      if (onError) {
        onError(error);
      }
    } finally {
      setLoading(false);
    }
  }, [endpoint, queryParams, enabled, onSuccess, onError]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  // Filter methods
  const setFilter = useCallback((key: string, value: any) => {
    setQueryParams((prev) => ({
      ...prev,
      filters: { ...prev.filters, [key]: value },
      pagination: { ...prev.pagination!, page: 1 }, // Reset to page 1
    }));
  }, []);

  const removeFilter = useCallback((key: string) => {
    setQueryParams((prev) => {
      const newFilters = { ...prev.filters };
      delete newFilters[key];
      return {
        ...prev,
        filters: newFilters,
        pagination: { ...prev.pagination!, page: 1 },
      };
    });
  }, []);

  const clearFilters = useCallback(() => {
    setQueryParams((prev) => ({
      ...prev,
      filters: {},
      pagination: { ...prev.pagination!, page: 1 },
    }));
  }, []);

  // Sort methods
  const setSort = useCallback((field: string, order: "asc" | "desc") => {
    setQueryParams((prev) => ({
      ...prev,
      sort: [{ field, order }],
    }));
  }, []);

  const addSort = useCallback((field: string, order: "asc" | "desc") => {
    setQueryParams((prev) => ({
      ...prev,
      sort: [...(prev.sort || []), { field, order }],
    }));
  }, []);

  const clearSort = useCallback(() => {
    setQueryParams((prev) => ({
      ...prev,
      sort: [],
    }));
  }, []);

  // Search methods
  const setSearch = useCallback((search: string) => {
    setQueryParams((prev) => ({
      ...prev,
      search,
      pagination: { ...prev.pagination!, page: 1 },
    }));
  }, []);

  const clearSearch = useCallback(() => {
    setQueryParams((prev) => ({
      ...prev,
      search: undefined,
      pagination: { ...prev.pagination!, page: 1 },
    }));
  }, []);

  // Pagination methods
  const setPage = useCallback((page: number) => {
    setQueryParams((prev) => ({
      ...prev,
      pagination: { ...prev.pagination!, page },
    }));
  }, []);

  const setLimit = useCallback((limit: number) => {
    setQueryParams((prev) => ({
      ...prev,
      pagination: { ...prev.pagination!, limit, page: 1 },
    }));
  }, []);

  const nextPage = useCallback(() => {
    setQueryParams((prev) => ({
      ...prev,
      pagination: {
        ...prev.pagination!,
        page: Math.min(prev.pagination!.page + 1, pagination.totalPages),
      },
    }));
  }, [pagination.totalPages]);

  const prevPage = useCallback(() => {
    setQueryParams((prev) => ({
      ...prev,
      pagination: {
        ...prev.pagination!,
        page: Math.max(prev.pagination!.page - 1, 1),
      },
    }));
  }, []);

  return {
    // Data
    data,
    loading,
    error,
    pagination,
    queryParams,

    // Methods
    refetch: fetchData,
    setFilter,
    removeFilter,
    clearFilters,
    setSort,
    addSort,
    clearSort,
    setSearch,
    clearSearch,
    setPage,
    setLimit,
    nextPage,
    prevPage,
  };
}
