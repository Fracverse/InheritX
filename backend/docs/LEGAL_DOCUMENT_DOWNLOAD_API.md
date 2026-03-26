# Legal Document Download API

## Overview

The Legal Document Download API provides secure endpoints for users to download their generated and signed legal will documents in PDF format. This API ensures that only authorized users can access their documents and maintains a complete audit trail of all download activities.

## Features

- **Secure Authentication**: All endpoints require valid JWT authentication
- **Authorization Checks**: Users can only download their own documents
- **Version Support**: Download specific versions of will documents
- **Audit Trail**: All downloads are logged as events for compliance
- **Proper HTTP Headers**: Documents are served with appropriate content-type and cache-control headers
- **PDF Format**: Documents are returned as downloadable PDF files

## Endpoints

### 1. Download Will Document by ID

Download a specific will document using its unique document ID.

**Endpoint**: `GET /api/will/documents/:document_id/download`

**Authentication**: Required (Bearer token)

**Path Parameters**:

- `document_id` (UUID): The unique identifier of the will document

**Response**:

- **Status**: 200 OK
- **Content-Type**: `application/pdf`
- **Headers**:
  - `Content-Disposition`: `attachment; filename="will_<plan_id>_<timestamp>.pdf"`
  - `Cache-Control`: `no-cache, no-store, must-revalidate`
  - `Pragma`: `no-cache`
  - `Expires`: `0`
- **Body**: Binary PDF content

**Error Responses**:

- `401 Unauthorized`: Missing or invalid authentication token
- `404 Not Found`: Document not found or user not authorized to access it

**Example**:

```bash
curl -X GET \
  'https://api.inheritx.com/api/will/documents/550e8400-e29b-41d4-a716-446655440000/download' \
  -H 'Authorization: Bearer <your_jwt_token>' \
  --output will_document.pdf
```

### 2. Download Will Document by Version

Download a specific version of a will document for a given plan.

**Endpoint**: `GET /api/plans/:plan_id/will/documents/:version/download`

**Authentication**: Required (Bearer token)

**Path Parameters**:

- `plan_id` (UUID): The unique identifier of the inheritance plan
- `version` (integer): The version number of the will document (starts at 1)

**Response**:

- **Status**: 200 OK
- **Content-Type**: `application/pdf`
- **Headers**:
  - `Content-Disposition`: `attachment; filename="will_<plan_id>_<timestamp>.pdf"`
  - `Cache-Control`: `no-cache, no-store, must-revalidate`
  - `Pragma`: `no-cache`
  - `Expires`: `0`
- **Body**: Binary PDF content

**Error Responses**:

- `401 Unauthorized`: Missing or invalid authentication token
- `404 Not Found`: Plan, version not found, or user not authorized to access it

**Example**:

```bash
curl -X GET \
  'https://api.inheritx.com/api/plans/550e8400-e29b-41d4-a716-446655440001/will/documents/1/download' \
  -H 'Authorization: Bearer <your_jwt_token>' \
  --output will_document_v1.pdf
```

## Security Features

### Authentication & Authorization

1. **JWT Authentication**: All requests must include a valid JWT token in the Authorization header
2. **User Ownership Verification**: The API verifies that the authenticated user owns the requested document
3. **Database-Level Security**: Queries include user_id filters to prevent unauthorized access

### Audit Trail

Every document download is logged as a `will_decrypted` event in the `will_event_log` table with the following information:

- Document ID
- Plan ID
- Vault ID
- User ID (who accessed the document)
- Timestamp

This provides a complete audit trail for compliance and security monitoring.

### Cache Control

Documents are served with strict cache-control headers to prevent caching of sensitive legal documents:

- `Cache-Control: no-cache, no-store, must-revalidate`
- `Pragma: no-cache`
- `Expires: 0`

## Use Cases

### 1. Download Latest Will Document

```javascript
// Get the latest document for a plan
const response = await fetch("/api/plans/{plan_id}/will/documents", {
  headers: {
    Authorization: `Bearer ${token}`,
  },
});
const documents = await response.json();
const latestDoc = documents.data[0]; // Documents are sorted by version DESC

// Download the latest document
window.location.href = `/api/will/documents/${latestDoc.document_id}/download`;
```

### 2. Download Specific Version

```javascript
// Download version 2 of a will document
const planId = "550e8400-e29b-41d4-a716-446655440001";
const version = 2;

const response = await fetch(
  `/api/plans/${planId}/will/documents/${version}/download`,
  {
    headers: {
      Authorization: `Bearer ${token}`,
    },
  },
);

const blob = await response.blob();
const url = window.URL.createObjectURL(blob);
const a = document.createElement("a");
a.href = url;
a.download = `will_v${version}.pdf`;
a.click();
```

### 3. Verify Document Before Download

```javascript
// First verify the document integrity
const verifyResponse = await fetch(`/api/will/documents/${documentId}/verify`, {
  headers: {
    Authorization: `Bearer ${token}`,
  },
});
const verification = await verifyResponse.json();

if (verification.data.is_valid) {
  // Document is valid, proceed with download
  window.location.href = `/api/will/documents/${documentId}/download`;
} else {
  console.error("Document integrity check failed");
}
```

## Integration with Other APIs

The download API works seamlessly with other will document APIs:

1. **Generate Document** → `POST /api/plans/:plan_id/will/generate`
2. **List Documents** → `GET /api/plans/:plan_id/will/documents`
3. **Verify Document** → `GET /api/will/documents/:document_id/verify`
4. **Download Document** → `GET /api/will/documents/:document_id/download`

## Event Logging

Downloads trigger a `will_decrypted` event that can be queried using:

```bash
# Get all events for a document
GET /api/will/documents/:document_id/events

# Get all events for a plan
GET /api/plans/:plan_id/will/events

# Get event statistics
GET /api/plans/:plan_id/will/events/stats
```

## Best Practices

1. **Always Verify Authentication**: Ensure the user is authenticated before attempting downloads
2. **Handle Errors Gracefully**: Provide clear error messages when downloads fail
3. **Monitor Download Events**: Track download patterns for security monitoring
4. **Version Management**: Keep track of which version users are downloading
5. **Secure Storage**: Store downloaded PDFs securely on the client side
6. **Compliance**: Maintain audit logs of all document access for legal compliance

## Error Handling

```javascript
async function downloadWillDocument(documentId, token) {
  try {
    const response = await fetch(`/api/will/documents/${documentId}/download`, {
      headers: {
        Authorization: `Bearer ${token}`,
      },
    });

    if (!response.ok) {
      if (response.status === 401) {
        throw new Error("Authentication required");
      } else if (response.status === 404) {
        throw new Error("Document not found or access denied");
      } else {
        throw new Error("Failed to download document");
      }
    }

    const blob = await response.blob();
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `will_document_${documentId}.pdf`;
    a.click();
    window.URL.revokeObjectURL(url);
  } catch (error) {
    console.error("Download failed:", error);
    // Show user-friendly error message
  }
}
```

## Testing

The API includes comprehensive test coverage:

- ✅ Successful document download
- ✅ Unauthorized access prevention
- ✅ Document not found handling
- ✅ Version-specific downloads
- ✅ Event logging verification
- ✅ Authentication requirement
- ✅ Multiple version downloads

Run tests with:

```bash
cargo test --test will_download_api_test
```

## Performance Considerations

1. **Base64 Decoding**: Documents are stored as base64 and decoded on-the-fly
2. **Database Queries**: Optimized with indexes on `document_id` and `user_id`
3. **Event Logging**: Asynchronous to avoid blocking the download response
4. **Memory Efficiency**: Streams large PDFs efficiently

## Future Enhancements

- [ ] Support for encrypted document downloads
- [ ] Batch download of multiple versions
- [ ] Download as ZIP archive with metadata
- [ ] Watermarking for draft documents
- [ ] Digital signature verification before download
- [ ] Rate limiting for download endpoints
- [ ] Download expiry links for sharing

## Related Documentation

- [Will PDF Generator](./WILL_PDF_GENERATOR.md)
- [Will Event Logging](./EVENTS.md)
- [Document Verification](./DOCUMENT_VERIFICATION.md)
- [Authentication](./AUTHENTICATION.md)
