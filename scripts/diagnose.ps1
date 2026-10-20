# Rengo Diagnostic Script
# Helps identify common issues

Write-Host "🔍 Rengo Diagnostics" -ForegroundColor Cyan
Write-Host "=========================" -ForegroundColor Cyan

# Check Rust installation
Write-Host "`n📦 Checking Rust installation..." -ForegroundColor Yellow
try {
    $cargoVersion = cargo --version 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Cargo found: $cargoVersion" -ForegroundColor Green
    } else {
        Write-Host "❌ Cargo not found or not working" -ForegroundColor Red
    }
} catch {
    Write-Host "❌ Cargo not found or not working" -ForegroundColor Red
}

# Check rustc
try {
    $rustcVersion = rustc --version 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Rust compiler found: $rustcVersion" -ForegroundColor Green
    } else {
        Write-Host "❌ Rust compiler not found" -ForegroundColor Red
    }
} catch {
    Write-Host "❌ Rust compiler not found" -ForegroundColor Red
}

# Check Visual Studio Build Tools (on Windows)
Write-Host "`n🔧 Checking Visual Studio Build Tools..." -ForegroundColor Yellow
try {
    $linkExists = Get-Command link.exe -ErrorAction SilentlyContinue
    if ($linkExists) {
        Write-Host "✅ MSVC linker found" -ForegroundColor Green
    } else {
        Write-Host "❌ MSVC linker not found" -ForegroundColor Red
        Write-Host "   Install Visual Studio Build Tools with C++ support" -ForegroundColor Yellow
    }
} catch {
    Write-Host "❌ MSVC linker not found" -ForegroundColor Red
}

# Check current directory
Write-Host "`n📁 Checking project structure..." -ForegroundColor Yellow
if (Test-Path "Cargo.toml") {
    Write-Host "✅ Cargo.toml found" -ForegroundColor Green
} else {
    Write-Host "❌ Cargo.toml not found" -ForegroundColor Red
}

if (Test-Path "Cargo.lock") {
    Write-Host "✅ Cargo.lock found" -ForegroundColor Green
} else {
    Write-Host "❌ Cargo.lock not found" -ForegroundColor Red
}

# Check workspace members
Write-Host "`n🔗 Checking workspace members..." -ForegroundColor Yellow
$workspaceMembers = @("rengo-core", "rengo-config", "rengo-storage")
foreach ($member in $workspaceMembers) {
    if (Test-Path "$member/Cargo.toml") {
        Write-Host "✅ $member/Cargo.toml found" -ForegroundColor Green
    } else {
        Write-Host "❌ $member/Cargo.toml not found" -ForegroundColor Red
    }
}

# Basic cargo check
Write-Host "`n🧪 Testing basic cargo check..." -ForegroundColor Yellow
try {
    $result = cargo check --message-format=short 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Basic cargo check passed" -ForegroundColor Green
    } else {
        Write-Host "❌ Basic cargo check failed" -ForegroundColor Red
        Write-Host "Error output:" -ForegroundColor Yellow
        Write-Host $result -ForegroundColor Red
    }
} catch {
    Write-Host "❌ Basic cargo check failed with exception" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
}

Write-Host "`n📋 Recommendations:" -ForegroundColor Cyan
Write-Host "- Ensure Visual Studio Build Tools are installed with C++ support" -ForegroundColor White
Write-Host "- Try running: rustup update" -ForegroundColor White
Write-Host "- If issues persist, try: cargo clean && cargo update" -ForegroundColor White
Write-Host "- For CI issues, check GitHub Actions logs" -ForegroundColor White
