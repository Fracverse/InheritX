use inheritx_backend::legacy_content::{LegacyContentService, UploadMetadata};
use sqlx::PgPool;


mod helpers;

#[sqlx::test]
async fn test_validate_video_types(_pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    assert!(LegacyContentService::validate_content_type("video/mp4").is_ok());
    assert!(LegacyContentService::validate_content_type("video/webm").is_ok());
    assert!(LegacyContentService::validate_content_type("video/mpeg").is_ok());
    Ok(())
}

#[sqlx::test]
async fn test_validate_audio_types(_pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    assert!(LegacyContentService::validate_content_type("audio/mpeg").is_ok());
    assert!(LegacyContentService::validate_content_type("audio/wav").is_ok());
    assert!(LegacyContentService::validate_content_type("audio/ogg").is_ok());
    Ok(())
}

#[sqlx::test]
async fn test_validate_text_types(_pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    assert!(LegacyContentService::validate_content_type("text/plain").is_ok());
    assert!(LegacyContentService::validate_content_type("text/markdown").is_ok());
    Ok(())
}

#[sqlx::test]
async fn test_validate_document_types(_pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    assert!(LegacyContentService::validate_content_type("application/pdf").is_ok());
    assert!(LegacyContentService::validate_content_type("application/msword").is_ok());
    assert!(LegacyContentService::validate_content_type(
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
    )
    .is_ok());
    Ok(())
}

#[sqlx::test]
async fn test_reject_invalid_types(_pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    assert!(LegacyContentService::validate_content_type("application/exe").is_err());
    assert!(LegacyContentService::validate_content_type("image/png").is_err());
    assert!(LegacyContentService::validate_content_type("video/invalid").is_err());
    Ok(())
}

#[sqlx::test]
async fn test_validate_file_size(_pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    assert!(LegacyContentService::validate_file_size(1024).is_ok());
    assert!(LegacyContentService::validate_file_size(1024 * 1024).is_ok());
    assert!(LegacyContentService::validate_file_size(524_288_000).is_ok()); // 500MB
    Ok(())
}

#[sqlx::test]
async fn test_reject_oversized_files(_pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    assert!(LegacyContentService::validate_file_size(524_288_001).is_err()); // Over 500MB
    assert!(LegacyContentService::validate_file_size(0).is_err()); // Empty file
    Ok(())
}

#[sqlx::test]
async fn test_file_hash_calculation(_pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let content1 = b"test content";
    let content2 = b"test content";
    let content3 = b"different content";

    let hash1 = LegacyContentService::calculate_file_hash(content1);
    let hash2 = LegacyContentService::calculate_file_hash(content2);
    let hash3 = LegacyContentService::calculate_file_hash(content3);

    assert_eq!(hash1, hash2, "Same content should produce same hash");
    assert_ne!(
        hash1, hash3,
        "Different content should produce different hash"
    );
    assert_eq!(hash1.len(), 64, "SHA-256 hash should be 64 hex characters");
    Ok(())
}

#[sqlx::test]
async fn test_create_content_record(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let user_id = helpers::create_test_user(&pool, "uploader@test.com").await?;

    let metadata = UploadMetadata {
        original_filename: "test_video.mp4".to_string(),
        content_type: "video/mp4".to_string(),
        file_size: 1024 * 1024, // 1MB
        description: Some("Test video upload".to_string()),
    };

    let storage_path = "legacy_content/test/path/file.mp4".to_string();
    let file_hash = "abc123def456".to_string();

    let record = LegacyContentService::create_content_record(
        &pool,
        user_id,
        &metadata,
        storage_path.clone(),
        file_hash.clone(),
    )
    .await
    .expect("Failed to create content record");

    assert_eq!(record.owner_user_id, user_id);
    assert_eq!(record.original_filename, "test_video.mp4");
    assert_eq!(record.content_type, "video/mp4");
    assert_eq!(record.file_size, 1024 * 1024);
    assert_eq!(record.storage_path, storage_path);
    assert_eq!(record.file_hash, file_hash);
    assert_eq!(record.status, "active");

    Ok(())
}

#[sqlx::test]
async fn test_list_user_content(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let user_id = helpers::create_test_user(&pool, "user1@test.com").await?;
    let other_user_id = helpers::create_test_user(&pool, "user2@test.com").await?;

    // Create content for user1
    for i in 1..=3 {
        let metadata = UploadMetadata {
            original_filename: format!("file{}.mp4", i),
            content_type: "video/mp4".to_string(),
            file_size: 1024 * i,
            description: None,
        };

        LegacyContentService::create_content_record(
            &pool,
            user_id,
            &metadata,
            format!("path/file{}.mp4", i),
            format!("hash{}", i),
        )
        .await?;
    }

    // Create content for user2
    let metadata = UploadMetadata {
        original_filename: "other.mp4".to_string(),
        content_type: "video/mp4".to_string(),
        file_size: 2048,
        description: None,
    };

    LegacyContentService::create_content_record(
        &pool,
        other_user_id,
        &metadata,
        "path/other.mp4".to_string(),
        "hash_other".to_string(),
    )
    .await?;

    // List user1's content
    let filters = inheritx_backend::legacy_content::ContentListFilters {
        content_type_prefix: None,
        limit: None,
        offset: None,
    };

    let content = LegacyContentService::list_user_content(&pool, user_id, &filters)
        .await
        .expect("Failed to list content");

    assert_eq!(content.len(), 3, "User should see only their own content");
    assert!(content.iter().all(|c| c.owner_user_id == user_id));

    Ok(())
}

#[sqlx::test]
async fn test_filter_by_content_type(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let user_id = helpers::create_test_user(&pool, "user3@test.com").await?;

    // Create video content
    let video_meta = UploadMetadata {
        original_filename: "video.mp4".to_string(),
        content_type: "video/mp4".to_string(),
        file_size: 1024,
        description: None,
    };
    LegacyContentService::create_content_record(
        &pool,
        user_id,
        &video_meta,
        "path/video.mp4".to_string(),
        "hash1".to_string(),
    )
    .await?;

    // Create audio content
    let audio_meta = UploadMetadata {
        original_filename: "audio.mp3".to_string(),
        content_type: "audio/mpeg".to_string(),
        file_size: 512,
        description: None,
    };
    LegacyContentService::create_content_record(
        &pool,
        user_id,
        &audio_meta,
        "path/audio.mp3".to_string(),
        "hash2".to_string(),
    )
    .await?;

    // Filter by video
    let video_filters = inheritx_backend::legacy_content::ContentListFilters {
        content_type_prefix: Some("video/".to_string()),
        limit: None,
        offset: None,
    };

    let videos = LegacyContentService::list_user_content(&pool, user_id, &video_filters).await?;
    assert_eq!(videos.len(), 1);
    assert!(videos[0].content_type.starts_with("video/"));

    // Filter by audio
    let audio_filters = inheritx_backend::legacy_content::ContentListFilters {
        content_type_prefix: Some("audio/".to_string()),
        limit: None,
        offset: None,
    };

    let audios = LegacyContentService::list_user_content(&pool, user_id, &audio_filters).await?;
    assert_eq!(audios.len(), 1);
    assert!(audios[0].content_type.starts_with("audio/"));

    Ok(())
}

#[sqlx::test]
async fn test_get_content_by_id(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let user_id = helpers::create_test_user(&pool, "user4@test.com").await?;

    let metadata = UploadMetadata {
        original_filename: "document.pdf".to_string(),
        content_type: "application/pdf".to_string(),
        file_size: 2048,
        description: Some("Important document".to_string()),
    };

    let record = LegacyContentService::create_content_record(
        &pool,
        user_id,
        &metadata,
        "path/document.pdf".to_string(),
        "hash_doc".to_string(),
    )
    .await?;

    let retrieved = LegacyContentService::get_content_by_id(&pool, record.id, user_id)
        .await
        .expect("Failed to get content");

    assert_eq!(retrieved.id, record.id);
    assert_eq!(retrieved.original_filename, "document.pdf");

    Ok(())
}

#[sqlx::test]
async fn test_unauthorized_access_blocked(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let owner_id = helpers::create_test_user(&pool, "owner@test.com").await?;
    let other_user_id = helpers::create_test_user(&pool, "other@test.com").await?;

    let metadata = UploadMetadata {
        original_filename: "private.mp4".to_string(),
        content_type: "video/mp4".to_string(),
        file_size: 1024,
        description: None,
    };

    let record = LegacyContentService::create_content_record(
        &pool,
        owner_id,
        &metadata,
        "path/private.mp4".to_string(),
        "hash_private".to_string(),
    )
    .await?;

    // Other user should not be able to access
    let result = LegacyContentService::get_content_by_id(&pool, record.id, other_user_id).await;
    assert!(result.is_err(), "Other user should not access content");

    Ok(())
}

#[sqlx::test]
async fn test_delete_content(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let user_id = helpers::create_test_user(&pool, "user5@test.com").await?;

    let metadata = UploadMetadata {
        original_filename: "to_delete.mp4".to_string(),
        content_type: "video/mp4".to_string(),
        file_size: 1024,
        description: None,
    };

    let record = LegacyContentService::create_content_record(
        &pool,
        user_id,
        &metadata,
        "path/to_delete.mp4".to_string(),
        "hash_delete".to_string(),
    )
    .await?;

    // Delete content
    LegacyContentService::delete_content(&pool, record.id, user_id)
        .await
        .expect("Failed to delete content");

    // Should not be retrievable
    let result = LegacyContentService::get_content_by_id(&pool, record.id, user_id).await;
    assert!(result.is_err(), "Deleted content should not be retrievable");

    Ok(())
}

#[sqlx::test]
async fn test_storage_stats(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let user_id = helpers::create_test_user(&pool, "user6@test.com").await?;

    // Create various content types
    let contents = vec![
        ("video.mp4", "video/mp4", 1024 * 1024),
        ("audio.mp3", "audio/mpeg", 512 * 1024),
        ("doc.pdf", "application/pdf", 256 * 1024),
        ("text.txt", "text/plain", 10 * 1024),
    ];

    for (filename, content_type, size) in contents {
        let metadata = UploadMetadata {
            original_filename: filename.to_string(),
            content_type: content_type.to_string(),
            file_size: size,
            description: None,
        };

        LegacyContentService::create_content_record(
            &pool,
            user_id,
            &metadata,
            format!("path/{}", filename),
            format!("hash_{}", filename),
        )
        .await?;
    }

    let stats = LegacyContentService::get_user_storage_stats(&pool, user_id)
        .await
        .expect("Failed to get stats");

    assert_eq!(stats.total_files, 4);
    assert_eq!(stats.video_count, 1);
    assert_eq!(stats.audio_count, 1);
    assert_eq!(stats.document_count, 1);
    assert_eq!(stats.text_count, 1);
    assert!(stats.total_size > 0);

    Ok(())
}
