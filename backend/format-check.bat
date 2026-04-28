@echo off
REM Script to check and fix formatting before committing (Windows)
REM This prevents CI formatting failures

echo Checking Rust code formatting...

REM Check if cargo is installed
where cargo >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo Cargo is not installed!
    echo Install Rust from: https://rustup.rs/
    exit /b 1
)

REM Check formatting
echo Running cargo fmt --check...
cargo fmt --check
if %ERRORLEVEL% EQU 0 (
    echo Code is properly formatted!
    exit /b 0
) else (
    echo.
    echo Code formatting issues found!
    echo.
    set /p response="Would you like to auto-fix? (y/n): "
    
    if /i "%response%"=="y" (
        echo Fixing formatting...
        cargo fmt
        echo Formatting fixed! Please review changes and commit.
        exit /b 0
    ) else (
        echo Please run: cargo fmt
        exit /b 1
    )
)
