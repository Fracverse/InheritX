# Legacy Content Upload System

## Overview

The Legacy Content Upload feature allows InheritX users to securely upload and store various types of media files (video, audio, text, documents) as part of their digital inheritance. Files are validated, hashed, and stored with proper access controls.

## Supported File Types

### Video
- `video/mp4`
- `video/mpeg`
- `video/quicktime`
- `video/x-msvideo`
- `video/webm`

### Audio
- `audio/mpeg` (MP3)
- `audio/wav`
- `audio/ogg`
- `audio/mp4`
- `audio/webm`

### Text
- `text/plain`
- `text/markdown`
- `text/html`

### Documents
- `application/pdf`
- `application/msword` (DOC)
- `application/vnd.openxmlformats-officedocument.wordprocessingml.document` (DOCX)
- `application/vnd.ms-excel` (XLS)
- `application/vnd.openxmlformats-officedocument.spreadsheetml.sheet` (XLSX)

## File Size Limits

- **Maximum file size:** 500MB (524,288,000 bytes)
- **Minimum file size:** 1 byte (empty files rejected)

## Database Schema

```sql
CREATE TABLE legacy_content (
    id UUID PRIMARY KEY,
    owner_user_id UUID NOT NULL REFERENCES users(id),
    filename VARCHAR(255) NOT NULL,
    original_filename VARCHAR(255) NOT NULL,
    content_type VARCHAR(100) NOT NULL,
    file_size BIGINT NOT NULL,
    storage_path TEXT NOT NULL,
    file_hash VARCHAR(64) NOT NULL,
    encrypted BOOLEAN NOT NULL DEFAULT false,
    encryption_key_version INTEGER,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL
);
```

## API Endpoints

### Upload Content

```http
POST /api/content/upload
Authorization: Bearer <user_token>
Content-Type: multipart/form-data

Form Data:
- file: <binary file data>
- metadata: {
    "original_filename": "my_video.mp4",
    "content_type": "video/mp4",
    "file_size": 1048576,
    "description": "Family vacation video"
  }
```

**Response:**
```json
{
  "status": "success",
  "message": "File uploaded successfully",
  "data": {
    "id": "uuid",
    "owner_user_id": "uuid",
    "filename": "generated-uuid",
    "original_filename": "my_video.mp4",
    "content_type": "video/mp4",
    "file_size": 1048576,
    "storage_path": "legacy_content/user-id/2024/03/28/filename",
    "file_hash": "sha256-hash",
    "status": "active",
    "created_at": "2024-03-28T10:00:00Z"
  }
}
```

### List User Content

```http
GET /api/content?content_type_prefix=video/&limit=50&offset=0
Authorization: Bearer <user_token>
```

**Response:**
```json
{
  "status": "success",
  "data": [...],
  "count": 10
}
```

### Get Content by ID

```http
GET /api/content/:content_id
Authorization: Bearer <user_token>
```

### Download Content

```http
GET /api/content/:content_id/download
Authorization: Bearer <user_token>
```

Returns the file with appropriate headers for download.

### Delete Content

```http
DELETE /api/content/:content_id
Authorization: Bearer <user_token>
```

Performs soft delete (sets status to 'deleted').

### Get Storage Statistics

```http
GET /api/content/stats
Authorization: Bearer <user_token>
```

**Response:**
```json
{
  "status": "success",
  "data": {
    "total_files": 25,
    "total_size": 104857600,
    "video_count": 10,
    "audio_count": 8,
    "text_count": 5,
    "document_count": 2
  }
}
```

## Security Features

### ✅ File Type Validation
- Only whitelisted MIME types accepted
- Enforced at application level and database constraint level

### ✅ File Size Validation
- Maximum 500MB per file
- Prevents resource exhaustion attacks

### ✅ Access Control
- Users can only access their own content
- Database queries filtered by `owner_user_id`
- JWT authentication required

### ✅ File Integrity
- SHA-256 hash calculated for each file
- Stored in database for verification
- Detects file tampering

### ✅ Storage Path Isolation
- Files organized by user ID and date
- Prevents path traversal attacks
- Example: `legacy_content/{user_id}/2024/03/28/{filename}`

## Storage Architecture

### Filesystem Structure

```
storage/
└── legacy_content/
    └── {user_id}/
        └── {year}/
            └── {month}/
                └── {day}/
                    └── {generated-filename}
```

### File Naming
- Original filename preserved in database
- Generated UUID used for filesystem storage
- Prevents filename collisions

## Usage Examples

### Upload with cURL

```bash
curl -X POST http://localhost:3000/api/content/upload \
  -H "Authorization: Bearer <token>" \
  -F "file=@/path/to/video.mp4" \
  -F 'metadata={"original_filename":"video.mp4","content_type":"video/mp4","file_size":1048576,"description":"My video"}'
```

### Upload with JavaScript

```javascript
const formData = new FormData();
formData.append('file', fileInput.files[0]);
formData.append('metadata', JSON.stringify({
  original_filename: fileInput.files[0].name,
  content_type: fileInput.files[0].type,
  file_size: fileInput.files[0].size,
  description: 'Optional description'
}));

const response = await fetch('/api/content/upload', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`
  },
  body: formData
});
```

## Validation Rules

### Content Type Validation
```rust
LegacyContentService::validate_content_type(&content_type)?;
```

Rejects:
- Executable files (.exe, .sh, .bat)
- Image files (not in scope)
- Unknown MIME types

### File Size Validation
```rust
LegacyContentService::validate_file_size(file_size)?;
```

Rejects:
- Empty files (0 bytes)
- Files over 500MB

## Testing

Run the test suite:

```bash
cd backend
cargo test legacy_content_tests
```

### Test Coverage

- ✅ File type validation (video, audio, text, documents)
- ✅ Invalid type rejection
- ✅ File size validation
- ✅ Oversized file rejection
- ✅ File hash calculation
- ✅ Content record creation
- ✅ User content listing
- ✅ Content type filtering
- ✅ Access control enforcement
- ✅ Content deletion
- ✅ Storage statistics

## Error Handling

### Common Errors

| Error | Status Code | Description |
|-------|-------------|-------------|
| `Unsupported content type` | 400 | File type not allowed |
| `File size exceeds maximum` | 400 | File over 500MB |
| `File is empty` | 400 | 0-byte file |
| `No file provided` | 400 | Missing file in upload |
| `Content not found` | 404 | Invalid content ID or unauthorized |
| `Failed to write file` | 500 | Filesystem error |

## Performance Considerations

### File Upload
- Streaming upload supported via multipart
- No in-memory buffering of entire file
- Async I/O for filesystem operations

### File Download
- Streaming download
- Proper cache headers set
- Content-Disposition for browser download

### Database Queries
- Indexed on `owner_user_id` and `created_at`
- Indexed on `content_type` for filtering
- Indexed on `file_hash` for deduplication

## Future Enhancements

- [ ] File encryption at rest
- [ ] Duplicate file detection (by hash)
- [ ] Thumbnail generation for videos
- [ ] Transcoding for audio/video
- [ ] Cloud storage integration (S3, Azure Blob)
- [ ] Virus scanning integration
- [ ] Compression for large files
- [ ] Resumable uploads
- [ ] Batch upload support
- [ ] Content sharing with beneficiaries

## Configuration

### Storage Path

Set in application or use default:

```rust
let storage_service = FileStorageService::new(
    std::path::PathBuf::from("./storage")
);
```

### Environment Variables

```bash
# Optional: Custom storage path
STORAGE_PATH=/var/inheritx/storage
```

## Monitoring

### Metrics to Track

- Total files uploaded per day
- Total storage used per user
- Average file size
- Upload success/failure rate
- Most common file types
- Storage growth rate

### Logging

All operations logged with:
- User ID
- Content ID
- Operation type (upload, download, delete)
- Timestamp
- File size
- Content type

## Compliance

### Data Retention
- Active files: Retained indefinitely
- Deleted files: Soft deleted, retained for 90 days
- User deletion: All content permanently deleted

### GDPR Compliance
- Users can request deletion of all content
- Export functionality available
- Access logs maintained

## Troubleshooting

### Upload Fails

1. Check file size < 500MB
2. Verify content type is supported
3. Ensure storage directory is writable
4. Check disk space available

### Download Fails

1. Verify content exists in database
2. Check file exists on filesystem
3. Verify user has access rights
4. Check storage path is correct

### Storage Issues

1. Monitor disk space usage
2. Implement cleanup for deleted files
3. Consider archival strategy for old files
4. Set up alerts for storage thresholds

## References

- [Axum Multipart Documentation](https://docs.rs/axum/latest/axum/extract/struct.Multipart.html)
- [SHA-256 Hashing](https://docs.rs/sha2/latest/sha2/)
- [File System Operations](https://docs.rs/tokio/latest/tokio/fs/)
