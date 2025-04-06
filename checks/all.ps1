<#
.SYNOPSIS
Run local quality checks for the Rust project (Windows version).
#>

# Stop on errors
$ErrorActionPreference = "Stop"

# Check if we're running in PowerShell 5.1 or later
if ($PSVersionTable.PSVersion.Major -lt 5) {
    Write-Error "This script requires PowerShell 5.1 or later"
    exit 1
}

function Test-CommandExists {
    param($command)
    $exists = $null -ne (Get-Command $command -ErrorAction SilentlyContinue)
    if (-not $exists) {
        Write-Error "Command '$command' is required but not found. Please install it."
    }
    return $exists
}

# Check prerequisites
Write-Host "`n=== Checking prerequisites ===" -ForegroundColor Cyan
$prereqsOk = $true
$prereqsOk = $prereqsOk -and (Test-CommandExists "cargo")
$prereqsOk = $prereqsOk -and (Test-CommandExists "rustup")
$prereqsOk = $prereqsOk -and (Test-CommandExists "exiftool")

if (-not $prereqsOk) {
    exit 1
}

# Install rust components if missing
Write-Host "`n=== Ensuring Rust components ===" -ForegroundColor Cyan
rustup component add rustfmt
rustup component add clippy

# Environment variables
$env:CARGO_TERM_COLOR = "always"
$env:RUSTFLAGS = "-Dwarnings"

# Run checks
$checksPassed = $true

try {
    # Format check
    Write-Host "`n=== Checking formatting with rustfmt ===" -ForegroundColor Cyan
    cargo fmt --all
    if ($LASTEXITCODE -ne 0) {
        Write-Host "`nFormatting issues found. Run 'cargo fmt --all' to fix." -ForegroundColor Red
        $checksPassed = $false
    }

    # Clippy check
    Write-Host "`n=== Running Clippy checks ===" -ForegroundColor Cyan
    cargo clippy --all-targets --all-features -- -D warnings `
        -A clippy::missing_errors_doc `
        -A clippy::missing_panics_doc
    if ($LASTEXITCODE -ne 0) {
        $checksPassed = $false
    }

    # Build in release mode
    Write-Host "`n=== Building in release mode ===" -ForegroundColor Cyan
    cargo build --release --verbose
    if ($LASTEXITCODE -ne 0) {
        $checksPassed = $false
    }

    # Run tests
    Write-Host "`n=== Running tests ===" -ForegroundColor Cyan
    cargo test --verbose
    if ($LASTEXITCODE -ne 0) {
        $checksPassed = $false
    }

    # Documentation check
    Write-Host "`n=== Checking documentation ===" -ForegroundColor Cyan
    cargo doc --no-deps --document-private-items
    if ($LASTEXITCODE -ne 0) {
        $checksPassed = $false
    }
}
catch {
    Write-Host "`nError during checks: $_" -ForegroundColor Red
    $checksPassed = $false
}

# Final result
if ($checksPassed) {
    Write-Host "`n=== All checks passed! ===" -ForegroundColor Green
}
else {
    Write-Host "`n=== Some checks failed ===" -ForegroundColor Red
    exit 1
}