import { describe, it, expect } from "vitest";
import {
  parseQueryParams,
  applyFilters,
  applySearch,
  applySort,
  applyPagination,
  applyQueryParams,
  buildQueryString,
} from "@/lib/api/filtering";

describe("API Filtering Utilities", () => {
  const mockData = [
    { id: 1, name: "Alice", status: "active", age: 30, created_at: "2024-01-01" },
    { id: 2, name: "Bob", status: "inactive", age: 25, created_at: "2024-02-01" },
    { id: 3, name: "Charlie", status: "active", age: 35, created_at: "2024-01-15" },
    { id: 4, name: "David", status: "pending", age: 28, created_at: "2024-03-01" },
  ];

  describe("parseQueryParams", () => {
    it("should parse filter parameters", () => {
      const params = new URLSearchParams("status=active&age=30");
      const result = parseQueryParams(params);

      expect(result.filters).toEqual({ status: "active", age: "30" });
    });

    it("should parse sort parameters", () => {
      const params = new URLSearchParams("sort=name:asc,age:desc");
      const result = parseQueryParams(params);

      expect(result.sort).toEqual([
        { field: "name", order: "asc" },
        { field: "age", order: "desc" },
      ]);
    });

    it("should parse search parameter", () => {
      const params = new URLSearchParams("search=alice");
      const result = parseQueryParams(params);

      expect(result.search).toBe("alice");
    });

    it("should parse pagination parameters", () => {
      const params = new URLSearchParams("page=2&limit=20");
      const result = parseQueryParams(params);

      expect(result.pagination).toEqual({ page: 2, limit: 20 });
    });

    it("should handle array filter values", () => {
      const params = new URLSearchParams("status=active&status=pending");
      const result = parseQueryParams(params);

      expect(result.filters?.status).toEqual(["active", "pending"]);
    });
  });

  describe("applyFilters", () => {
    it("should filter by single field", () => {
      const result = applyFilters(mockData, { status: "active" });

      expect(result).toHaveLength(2);
      expect(result.every((item) => item.status === "active")).toBe(true);
    });

    it("should filter by multiple fields", () => {
      const result = applyFilters(mockData, { status: "active", age: "30" });

      expect(result).toHaveLength(1);
      expect(result[0].name).toBe("Alice");
    });

    it("should handle array filters (OR logic)", () => {
      const result = applyFilters(mockData, { status: ["active", "pending"] });

      expect(result).toHaveLength(3);
    });

    it("should return all data when no filters", () => {
      const result = applyFilters(mockData, {});

      expect(result).toHaveLength(4);
    });
  });

  describe("applySearch", () => {
    it("should search across all fields", () => {
      const result = applySearch(mockData, "alice");

      expect(result).toHaveLength(1);
      expect(result[0].name).toBe("Alice");
    });

    it("should search specific fields", () => {
      const result = applySearch(mockData, "active", ["status"]);

      expect(result).toHaveLength(2);
    });

    it("should be case-insensitive", () => {
      const result = applySearch(mockData, "ALICE");

      expect(result).toHaveLength(1);
    });

    it("should return all data when no search term", () => {
      const result = applySearch(mockData, "");

      expect(result).toHaveLength(4);
    });
  });

  describe("applySort", () => {
    it("should sort ascending", () => {
      const result = applySort(mockData, [{ field: "age", order: "asc" }]);

      expect(result[0].age).toBe(25);
      expect(result[3].age).toBe(35);
    });

    it("should sort descending", () => {
      const result = applySort(mockData, [{ field: "age", order: "desc" }]);

      expect(result[0].age).toBe(35);
      expect(result[3].age).toBe(25);
    });

    it("should sort by multiple fields", () => {
      const result = applySort(mockData, [
        { field: "status", order: "asc" },
        { field: "age", order: "desc" },
      ]);

      expect(result[0].status).toBe("active");
      expect(result[0].age).toBe(35); // Charlie
    });

    it("should sort strings alphabetically", () => {
      const result = applySort(mockData, [{ field: "name", order: "asc" }]);

      expect(result[0].name).toBe("Alice");
      expect(result[3].name).toBe("David");
    });
  });

  describe("applyPagination", () => {
    it("should paginate correctly", () => {
      const result = applyPagination(mockData, { page: 1, limit: 2 });

      expect(result.data).toHaveLength(2);
      expect(result.total).toBe(4);
      expect(result.totalPages).toBe(2);
    });

    it("should handle second page", () => {
      const result = applyPagination(mockData, { page: 2, limit: 2 });

      expect(result.data).toHaveLength(2);
      expect(result.page).toBe(2);
    });

    it("should handle last page with fewer items", () => {
      const result = applyPagination(mockData, { page: 2, limit: 3 });

      expect(result.data).toHaveLength(1);
    });
  });

  describe("applyQueryParams", () => {
    it("should apply all operations", () => {
      const result = applyQueryParams(
        mockData,
        {
          filters: { status: "active" },
          sort: [{ field: "age", order: "desc" }],
          pagination: { page: 1, limit: 10 },
        },
        ["name", "status"]
      );

      expect(result.data).toHaveLength(2);
      expect(result.data[0].age).toBe(35); // Charlie
      expect(result.total).toBe(2);
    });

    it("should combine filter and search", () => {
      const result = applyQueryParams(
        mockData,
        {
          filters: { status: "active" },
          search: "charlie",
          pagination: { page: 1, limit: 10 },
        },
        ["name"]
      );

      expect(result.data).toHaveLength(1);
      expect(result.data[0].name).toBe("Charlie");
    });
  });

  describe("buildQueryString", () => {
    it("should build query string from params", () => {
      const queryString = buildQueryString({
        filters: { status: "active", age: "30" },
        sort: [{ field: "name", order: "asc" }],
        search: "test",
        pagination: { page: 2, limit: 20 },
      });

      expect(queryString).toContain("status=active");
      expect(queryString).toContain("age=30");
      expect(queryString).toContain("sort=name%3Aasc");
      expect(queryString).toContain("search=test");
      expect(queryString).toContain("page=2");
      expect(queryString).toContain("limit=20");
    });

    it("should handle array filters", () => {
      const queryString = buildQueryString({
        filters: { status: ["active", "pending"] },
        pagination: { page: 1, limit: 10 },
      });

      expect(queryString).toContain("status=active");
      expect(queryString).toContain("status=pending");
    });
  });
});
