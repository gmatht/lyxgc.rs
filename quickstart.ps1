# lyxgc chktex quickstart - fetch prebuilt binary and run on sample.
# Usage: .\quickstart.ps1
# Requires: PowerShell 5.1+, internet
$ErrorActionPreference = "Stop"
$REPO = "gmatht/lyxgc.rs"

# Resolve latest release and asset URL for Windows x64
$api = "https://api.github.com/repos/$REPO/releases/latest"
Write-Host "Fetching latest release..."
try {
    $release = Invoke-RestMethod -Uri $api -Headers @{ Accept = "application/vnd.github.v3+json" } -UseBasicParsing
} catch {
    Write-Host "No release found. Build from source: cd rs && cargo build --release"
    exit 1
}
$tag = $release.tag_name
$asset = $release.assets | Where-Object { $_.name -match "windows.*x64.*\.zip" } | Select-Object -First 1
if (-not $asset) {
    Write-Host "No Windows binary in release $tag"
    exit 1
}

# Download and extract
$dir = Join-Path $env:TEMP "lyxgc-chktex-$tag"
if (Test-Path $dir) { Remove-Item -Recurse -Force $dir }
New-Item -ItemType Directory -Path $dir | Out-Null
$zipPath = Join-Path $dir "bin.zip"
Write-Host "Downloading $($asset.name)..."
Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $zipPath -UseBasicParsing
Expand-Archive -Path $zipPath -DestinationPath $dir -Force

# Find chktex.exe
$chktex = Get-ChildItem -Path $dir -Recurse -Filter "chktex.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $chktex) {
    Write-Host "chktex.exe not found in archive"
    exit 1
}

# Sample LaTeX
$samplePath = Join-Path $dir "sample.tex"
@"
\documentclass{article}
\begin{document}
This is we that wrong.
Empty math: $$ $$ here.
\end{document}
"@ | Set-Content -Path $samplePath -Encoding UTF8

# Run
Write-Host ""
Write-Host "Running chktex on sample..." -ForegroundColor Cyan
& $chktex.FullName $samplePath
Write-Host ""
Write-Host "Binary: $($chktex.FullName)" -ForegroundColor Green
Write-Host "Add to PATH or copy to a permanent location."
Write-Host "Try: chktex yourfile.tex"
