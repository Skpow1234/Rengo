# Rengo Quick Fix Script
# Addresses common issues automatically

param(
    [switch]$Force
)

Write-Host "🔧 Rengo Quick Fix" -ForegroundColor Cyan
Write-Host "=======================" -ForegroundColor Cyan

# Clean and update
Write-Host "`n🧹 Cleaning old build artifacts..." -ForegroundColor Yellow
cargo clean

Write-Host "📦 Updating dependencies..." -ForegroundColor Yellow
cargo update

# Format code
Write-Host "`n🔧 Formatting code..." -ForegroundColor Yellow
cargo fmt

# Try basic check
Write-Host "`n🧪 Running basic check..." -ForegroundColor Yellow
try {
    cargo check
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Basic compilation successful!" -ForegroundColor Green
    } else {
        Write-Host "⚠️  Compilation failed, but this might be due to missing system dependencies" -ForegroundColor Yellow
        Write-Host "   This is normal on systems without Visual Studio Build Tools" -ForegroundColor White
    }
} catch {
    Write-Host "❌ Check failed with exception: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "`n📋 Next steps:" -ForegroundColor Cyan
Write-Host "- If you're on Windows, install Visual Studio Build Tools" -ForegroundColor White
Write-Host "- If you're on Linux/macOS, install clang and SSL dev libraries" -ForegroundColor White
Write-Host "- Run '.\scripts\diagnose.ps1' to check your setup" -ForegroundColor White
Write-Host "- Push changes to test CI pipeline" -ForegroundColor White
