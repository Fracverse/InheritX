@echo off
REM Storage Directory Setup Script for Legacy Content Uploads (Windows)

echo Setting up storage directories for InheritX Legacy Content...
echo.

REM Create base storage directory
if not exist "storage\legacy_content" (
    mkdir storage\legacy_content
    echo Created: storage\legacy_content
) else (
    echo Directory already exists: storage\legacy_content
)

echo.
echo ✅ Storage directories ready!
echo.
echo Directory structure:
echo   storage\
echo   └── legacy_content\
echo.
echo Files will be organized as:
echo   storage\legacy_content\{user_id}\{year}\{month}\{day}\{filename}
echo.
echo Next steps:
echo   1. Run migrations: cd backend ^&^& sqlx migrate run
echo   2. Run tests: cargo test legacy_content_tests
echo   3. Start backend: cargo run
echo.
pause
