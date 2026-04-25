use base64::Engine;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Standard pagination query parameters for offset-based pagination
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationQuery {
    /// Page number (1-indexed)
    pub page: Option<u32>,
    /// Number of items per page (default: 20, max: 100)
    pub limit: Option<u32>,
}

/// Cursor-based pagination query parameters
#[derive(Debug, Clone, Deserialize)]
pub struct CursorPaginationQuery {
    /// Cursor for the next page (opaque string)
    pub cursor: Option<String>,
    /// Number of items per page (default: 20, max: 100)
    pub limit: Option<u32>,
}

/// Standard pagination metadata response
#[derive(Debug, Clone, Serialize)]
pub struct PaginationMeta {
    /// Current page number
    pub page: u32,
    /// Items per page
    pub limit: u32,
    /// Total number of items
    pub total_count: i64,
    /// Total number of pages
    pub total_pages: u32,
    /// Whether there is a next page
    pub has_next: bool,
    /// Whether there is a previous page
    pub has_prev: bool,
}

/// Cursor-based pagination metadata response
#[derive(Debug, Clone, Serialize)]
pub struct CursorPaginationMeta {
    /// Number of items per page
    pub limit: u32,
    /// Total number of items (optional, can be expensive to compute)
    pub total_count: Option<i64>,
    /// Whether there is a next page
    pub has_next: bool,
    /// Cursor for the next page
    pub next_cursor: Option<String>,
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResponse<T> {
    pub status: String,
    pub data: Vec<T>,
    #[serde(flatten)]
    pub pagination: PaginationMeta,
}

/// Cursor-based paginated response wrapper
#[derive(Debug, Clone, Serialize)]
pub struct CursorPaginatedResponse<T> {
    pub status: String,
    pub data: Vec<T>,
    #[serde(flatten)]
    pub pagination: CursorPaginationMeta,
}

impl PaginationQuery {
    /// Normalize and validate pagination parameters
    /// Returns (page, limit, offset)
    pub fn normalize(&self) -> (u32, u32, i64) {
        let page = self.page.unwrap_or(1).max(1);
        let limit = self.limit.unwrap_or(20).clamp(1, 100);
        let offset = ((page - 1) * limit) as i64;
        (page, limit, offset)
    }

    /// Create pagination metadata from query and total count
    pub fn create_meta(&self, total_count: i64) -> PaginationMeta {
        let (page, limit, _) = self.normalize();
        let total_pages = ((total_count as f64) / (limit as f64)).ceil() as u32;
        let has_next = page < total_pages;
        let has_prev = page > 1;

        PaginationMeta {
            page,
            limit,
            total_count,
            total_pages,
            has_next,
            has_prev,
        }
    }

    /// Create a paginated response
    pub fn create_response<T>(&self, data: Vec<T>, total_count: i64) -> PaginatedResponse<T> {
        PaginatedResponse {
            status: "success".to_string(),
            data,
            pagination: self.create_meta(total_count),
        }
    }
}

impl CursorPaginationQuery {
    /// Normalize and validate cursor pagination parameters
    /// Returns (limit, decoded_cursor)
    pub fn normalize(&self) -> (u32, Option<DecodedCursor>) {
        let limit = self.limit.unwrap_or(20).clamp(1, 100);
        let decoded_cursor = self.cursor.as_ref().and_then(|c| decode_cursor(c).ok());
        (limit, decoded_cursor)
    }

    /// Create cursor pagination metadata
    pub fn create_meta(
        &self,
        has_next: bool,
        next_cursor: Option<String>,
        total_count: Option<i64>,
    ) -> CursorPaginationMeta {
        let limit = self.limit.unwrap_or(20).clamp(1, 100);

        CursorPaginationMeta {
            limit,
            total_count,
            has_next,
            next_cursor,
        }
    }

    /// Create a cursor-based paginated response
    pub fn create_response<T>(
        &self,
        data: Vec<T>,
        has_next: bool,
        next_cursor: Option<String>,
        total_count: Option<i64>,
    ) -> CursorPaginatedResponse<T> {
        CursorPaginatedResponse {
            status: "success".to_string(),
            data,
            pagination: self.create_meta(has_next, next_cursor, total_count),
        }
    }
}

/// Decoded cursor structure
#[derive(Debug, Clone)]
pub struct DecodedCursor {
    pub id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Encode a cursor from ID and timestamp
pub fn encode_cursor(id: Uuid, timestamp: chrono::DateTime<chrono::Utc>) -> String {
    let cursor_data = format!("{}|{}", id, timestamp.timestamp_millis());
    base64::engine::general_purpose::STANDARD.encode(cursor_data.as_bytes())
}

/// Decode a cursor to ID and timestamp
pub fn decode_cursor(cursor: &str) -> Result<DecodedCursor, String> {
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(cursor)
        .map_err(|e| format!("Invalid cursor: {}", e))?;
    let cursor_str = String::from_utf8(decoded).map_err(|e| format!("Invalid cursor: {}", e))?;

    let parts: Vec<&str> = cursor_str.split('|').collect();
    if parts.len() != 2 {
        return Err("Invalid cursor format".to_string());
    }

    let id = Uuid::parse_str(parts[0]).map_err(|e| format!("Invalid cursor ID: {}", e))?;
    let timestamp_millis: i64 = parts[1]
        .parse()
        .map_err(|e| format!("Invalid cursor timestamp: {}", e))?;
    let timestamp = chrono::DateTime::from_timestamp_millis(timestamp_millis)
        .ok_or_else(|| "Invalid timestamp".to_string())?;

    Ok(DecodedCursor { id, timestamp })
}

/// Helper to count total records for a table with optional filters
pub async fn count_records(
    db: &PgPool,
    table: &str,
    where_clause: Option<&str>,
    params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Sync)],
) -> Result<i64, sqlx::Error> {
    let query = if let Some(where_clause) = where_clause {
        format!("SELECT COUNT(*) FROM {} WHERE {}", table, where_clause)
    } else {
        format!("SELECT COUNT(*) FROM {}", table)
    };

    let mut query_builder = sqlx::query_scalar::<_, i64>(&query);
    for param in params {
        query_builder = query_builder.bind(*param);
    }

    query_builder.fetch_one(db).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_query_normalize_defaults() {
        let query = PaginationQuery {
            page: None,
            limit: None,
        };
        let (page, limit, offset) = query.normalize();
        assert_eq!(page, 1);
        assert_eq!(limit, 20);
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_pagination_query_normalize_custom() {
        let query = PaginationQuery {
            page: Some(3),
            limit: Some(50),
        };
        let (page, limit, offset) = query.normalize();
        assert_eq!(page, 3);
        assert_eq!(limit, 50);
        assert_eq!(offset, 100);
    }

    #[test]
    fn test_pagination_query_normalize_max_limit() {
        let query = PaginationQuery {
            page: Some(1),
            limit: Some(200),
        };
        let (page, limit, offset) = query.normalize();
        assert_eq!(page, 1);
        assert_eq!(limit, 100); // Clamped to max
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_pagination_query_normalize_zero_page() {
        let query = PaginationQuery {
            page: Some(0),
            limit: Some(10),
        };
        let (page, limit, offset) = query.normalize();
        assert_eq!(page, 1); // Minimum page is 1
        assert_eq!(limit, 10);
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_pagination_meta_calculation() {
        let query = PaginationQuery {
            page: Some(2),
            limit: Some(20),
        };
        let meta = query.create_meta(45);
        assert_eq!(meta.page, 2);
        assert_eq!(meta.limit, 20);
        assert_eq!(meta.total_count, 45);
        assert_eq!(meta.total_pages, 3);
        assert!(meta.has_next);
        assert!(meta.has_prev);
    }

    #[test]
    fn test_cursor_encode_decode() {
        let id = Uuid::new_v4();
        let timestamp = chrono::Utc::now();
        let cursor = encode_cursor(id, timestamp);
        let decoded = decode_cursor(&cursor).unwrap();
        assert_eq!(decoded.id, id);
        assert_eq!(
            decoded.timestamp.timestamp_millis(),
            timestamp.timestamp_millis()
        );
    }

    #[test]
    fn test_cursor_decode_invalid() {
        assert!(decode_cursor("invalid").is_err());
        assert!(decode_cursor("aW52YWxpZA==").is_err()); // "invalid" in base64
    }

    #[test]
    fn test_cursor_pagination_query_normalize() {
        let query = CursorPaginationQuery {
            cursor: None,
            limit: Some(50),
        };
        let (limit, cursor) = query.normalize();
        assert_eq!(limit, 50);
        assert!(cursor.is_none());
    }
}
