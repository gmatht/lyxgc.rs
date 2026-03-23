# lyxgc chktex quickstart - fetch prebuilt binary and run on sample.
# Optional: run LyX GUI setup directly from this rs script.
# Usage:
#   .\quickstart.ps1              # default: includes LyX GUI setup flow
#   .\quickstart.ps1 -NoLyXGui    # binary-only smoke test
# Requires: PowerShell 5.1+, internet
param(
    [switch]$NoLyXGui,
    [switch]$AdminPhase,
    [string]$AdminLogPath = ""
)

$ErrorActionPreference = "Stop"
$REPO = "gmatht/lyxgc.rs"
$script:InstallFailures = New-Object System.Collections.Generic.List[string]

function Show-InstallProgress {
    param(
        [int]$Percent,
        [string]$Status
    )
    Write-Progress -Activity "LyX GUI setup" -Status $Status -PercentComplete $Percent
    Write-Host ("[{0,3}%] {1}" -f $Percent, $Status)
}

function Start-LyXWithChktexPath {
    param(
        [string]$LyXExe,
        [string]$ChktexDir
    )
    $proc = New-Object System.Diagnostics.Process
    $proc.StartInfo = New-Object System.Diagnostics.ProcessStartInfo
    $proc.StartInfo.FileName = $LyXExe
    $proc.StartInfo.UseShellExecute = $false
    $proc.StartInfo.WorkingDirectory = Split-Path -Parent $LyXExe
    $proc.StartInfo.EnvironmentVariables["PATH"] = "$ChktexDir;$($env:PATH)"
    [void]$proc.Start()
}

function Setup-LyXUserDirWithChktex {
    param(
        [string]$ChktexExe
    )
    try {
        $userdir = Join-Path $HOME ".lyx-rs-chktex"
        if (-not (Test-Path $userdir)) {
            New-Item -ItemType Directory -Path $userdir | Out-Null
        }
        $chktexExeCfg = $ChktexExe.Replace("\", "/")
        # Keep warning IDs aligned with current py integration defaults.
        $cmd = "$chktexExeCfg -n1 -n3 -n6 -n9 -n22 -n25 -n30 -n38"
        $lyxrc = '\chktex_command "' + $cmd + '"' + "`n"
        Set-Content -Path (Join-Path $userdir "lyxrc") -Value $lyxrc -Encoding UTF8
        return $userdir
    } catch {
        return $null
    }
}

function Test-Administrator {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Add-InstallFailure {
    param([string]$Message)
    if ($Message) {
        $script:InstallFailures.Add($Message) | Out-Null
    }
}

function Write-AdminFailureLog {
    if (-not $AdminLogPath) { return }
    try {
        if ($script:InstallFailures.Count -eq 0) {
            "No dependency install failures recorded." | Set-Content -Path $AdminLogPath -Encoding UTF8
        } else {
            $script:InstallFailures | Set-Content -Path $AdminLogPath -Encoding UTF8
        }
    } catch {
        # Best-effort logging only.
    }
}

function Show-AdminFailureLogFromMain {
    param([string]$Path)
    if (-not $Path) { return }
    if (-not (Test-Path $Path -PathType Leaf)) { return }
    Write-Host ""
    Write-Host "Admin phase log:" -ForegroundColor Cyan
    try {
        Get-Content -Path $Path | ForEach-Object { Write-Host "  $_" }
    } finally {
        Remove-Item $Path -Force -ErrorAction SilentlyContinue
    }
}

function Test-NeedsAdminForMissingDeps {
    param(
        [bool]$MissingLyX,
        [bool]$MissingJava
    )
    if (Test-Administrator) { return $false }
    if (-not ($MissingLyX -or $MissingJava)) { return $false }

    $hasWinget = [bool](Get-Command winget -ErrorAction SilentlyContinue)
    $hasChoco = [bool](Get-Command choco -ErrorAction SilentlyContinue)

    # If winget is available, try user-scope installs without elevation first.
    if ($hasWinget) { return $false }
    # Chocolatey typically requires elevation.
    if ($hasChoco) { return $true }
    return $false
}

function Invoke-AdminDepsPhase {
    if (-not $PSCommandPath) {
        Write-Host "Cannot auto-elevate (script path unknown)." -ForegroundColor Yellow
        return $false
    }
    Write-Host "Administrator rights required for dependency install. Prompting UAC once..." -ForegroundColor Yellow
    $logPath = Join-Path $env:TEMP ("lyxgc-admin-phase-" + [guid]::NewGuid().ToString() + ".log")
    $args = @("-ExecutionPolicy", "Bypass", "-NoProfile", "-File", $PSCommandPath, "-AdminPhase", "-AdminLogPath", $logPath)
    try {
        $p = Start-Process powershell -ArgumentList $args -Verb RunAs -Wait -PassThru
        Show-AdminFailureLogFromMain -Path $logPath
        return ($p.ExitCode -eq 0)
    } catch {
        Write-Host "Elevation was cancelled or failed: $_" -ForegroundColor Yellow
        Show-AdminFailureLogFromMain -Path $logPath
        return $false
    }
}

function Find-LyX {
    $cmd = Get-Command "lyx" -ErrorAction SilentlyContinue
    if ($cmd -and $cmd.Source -and (Test-Path $cmd.Source -PathType Leaf)) { return $cmd.Source }

    $bases = @(
        $env:ProgramFiles,
        ${env:ProgramFiles(x86)},
        (Join-Path $env:LocalAppData "Programs"),
        "C:\Program Files",
        "C:\Program Files (x86)"
    ) | Where-Object { $_ }
    $versions = @("LyX 2.5", "LyX 2.4", "LyX 2.3", "LyX 2.2", "LyX 2.1")
    foreach ($base in $bases) {
        foreach ($ver in $versions) {
            $exe = Join-Path $base "$ver\bin\lyx.exe"
            if (Test-Path $exe -PathType Leaf) { return $exe }
        }
    }

    $chocoBin = "C:\ProgramData\chocolatey\bin\lyx.exe"
    if (Test-Path $chocoBin -PathType Leaf) { return $chocoBin }

    return $null
}

function Find-Java {
    $cmd = Get-Command "java" -ErrorAction SilentlyContinue
    if ($cmd -and $cmd.Source -and (Test-Path $cmd.Source -PathType Leaf)) { return $cmd.Source }

    if ($env:JAVA_HOME) {
        $javaFromHome = Join-Path $env:JAVA_HOME "bin\java.exe"
        if (Test-Path $javaFromHome -PathType Leaf) { return $javaFromHome }
    }

    $candidates = @()
    $candidates += (Join-Path $env:ProgramFiles "Eclipse Adoptium")
    $candidates += (Join-Path $env:ProgramFiles "Microsoft")
    $candidates += (Join-Path $env:ProgramFiles "Java")
    $candidates += (Join-Path $env:LocalAppData "Programs\Eclipse Adoptium")
    $candidates += (Join-Path $env:LocalAppData "Programs\Microsoft")
    $candidates += (Join-Path $env:LocalAppData "Programs\Java")

    foreach ($root in $candidates | Where-Object { $_ -and (Test-Path $_) }) {
        $javaExe = Get-ChildItem -Path $root -Recurse -Filter "java.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($javaExe) { return $javaExe.FullName }
    }
    return $null
}

function Find-LanguageTool {
    $paths = @(
        (Join-Path $env:LocalAppData "LanguageTool-stable"),
        (Join-Path $HOME ".data\LanguageTool-stable")
    )
    foreach ($p in $paths) {
        if (Test-Path $p) {
            $jar = Get-ChildItem -Path $p -Recurse -Filter "languagetool-commandline*.jar" -ErrorAction SilentlyContinue | Select-Object -First 1
            if ($jar) { return $jar.Directory.FullName }
        }
    }
    return $null
}

function Install-WithWingetOrChoco {
    param(
        [string]$Name,
        [string]$WingetId,
        [string]$ChocoId,
        [string]$FallbackUrl
    )
    if ($WingetId -and (Get-Command winget -ErrorAction SilentlyContinue)) {
        try {
            Write-Host "Installing $Name via winget (user scope)..."
            & winget install -e --id $WingetId --scope user --accept-package-agreements --accept-source-agreements
            if ($LASTEXITCODE -eq 0) { return $true }
        } catch {
            Write-Host "winget install for $Name failed: $_"
            Add-InstallFailure "${Name}: winget user-scope exception: $($_.Exception.Message)"
        }
        try {
            Write-Host "Installing $Name via winget (machine scope)..."
            & winget install -e --id $WingetId --scope machine --accept-package-agreements --accept-source-agreements
            if ($LASTEXITCODE -eq 0) { return $true }
        } catch {
            Write-Host "winget machine-scope install for $Name failed: $_"
            Add-InstallFailure "${Name}: winget machine-scope exception: $($_.Exception.Message)"
        }
    }
    if ($ChocoId -and (Get-Command choco -ErrorAction SilentlyContinue)) {
        if (-not (Test-Administrator)) {
            Write-Host "Skipping Chocolatey for $Name (requires Administrator)." -ForegroundColor Yellow
        } else {
        try {
            Write-Host "Installing $Name via Chocolatey..."
            & choco install $ChocoId -y --yes
            if ($LASTEXITCODE -eq 0) { return $true }
        } catch {
            Write-Host "choco install for $Name failed: $_"
            Add-InstallFailure "${Name}: Chocolatey exception: $($_.Exception.Message)"
        }
        }
    }
    Write-Host "$Name install requires manual setup: $FallbackUrl" -ForegroundColor Yellow
    Add-InstallFailure "${Name}: automatic install failed, manual setup required: $FallbackUrl"
    return $false
}

function Install-Java {
    # Prefer OpenJDK distributions first (Temurin/Microsoft), then legacy fallback.
    $attempts = @(
        @{ Winget = "EclipseAdoptium.Temurin.21.JDK"; Choco = "temurin21" },
        @{ Winget = "EclipseAdoptium.Temurin.21.JRE"; Choco = "temurin21jre" },
        @{ Winget = "Microsoft.OpenJDK.21"; Choco = "microsoft-openjdk" },
        @{ Winget = "Microsoft.OpenJDK.17"; Choco = "microsoft-openjdk17" },
        @{ Winget = "Oracle.JavaRuntimeEnvironment"; Choco = "jre8" }
    )
    foreach ($a in $attempts) {
        if (Install-WithWingetOrChoco -Name "Java" -WingetId $a.Winget -ChocoId $a.Choco -FallbackUrl "https://adoptium.net/") {
            return $true
        }
    }
    return $false
}

function Install-LanguageTool {
    $target = Join-Path $env:LocalAppData "LanguageTool-stable"
    $zipPath = Join-Path $env:TEMP "LanguageTool-stable.zip"
    $url = "https://languagetool.org/download/LanguageTool-stable.zip"
    try {
        Show-InstallProgress -Percent 68 -Status "Downloading LanguageTool"
        Write-Host "Downloading LanguageTool..."
        Invoke-WebRequest -Uri $url -OutFile $zipPath -UseBasicParsing
        Show-InstallProgress -Percent 76 -Status "Extracting LanguageTool"
        if (Test-Path $target) { Remove-Item -Recurse -Force $target }
        New-Item -ItemType Directory -Path $target | Out-Null
        Expand-Archive -Path $zipPath -DestinationPath $target -Force
        return $true
    } catch {
        Write-Host "LanguageTool install failed: $_" -ForegroundColor Yellow
        Write-Host "Install manually: https://languagetool.org/download/" -ForegroundColor Yellow
        Add-InstallFailure "LanguageTool: download/extract failed: $($_.Exception.Message)"
        return $false
    }
}

function Ensure-LyXGuiDeps {
    param(
        [switch]$AllowElevationRelaunch
    )
    Show-InstallProgress -Percent 40 -Status "Detecting LyX"
    $lyxExe = Find-LyX
    $javaExe = Find-Java

    if ($AllowElevationRelaunch) {
        $needAdmin = Test-NeedsAdminForMissingDeps -MissingLyX (-not $lyxExe) -MissingJava (-not $javaExe)
        if ($needAdmin) {
            Show-InstallProgress -Percent 44 -Status "Requesting UAC (single elevation)"
            [void](Invoke-AdminDepsPhase)
            Show-InstallProgress -Percent 48 -Status "Refreshing dependency detection"
            $lyxExe = Find-LyX
            $javaExe = Find-Java
        }
    }

    if (-not $lyxExe) {
        Show-InstallProgress -Percent 46 -Status "Installing LyX"
        Write-Host "LyX not found. Attempting install..."
        [void](Install-WithWingetOrChoco -Name "LyX" -WingetId "LyX.LyX" -ChocoId "lyx" -FallbackUrl "https://www.lyx.org/")
        Show-InstallProgress -Percent 52 -Status "Re-checking LyX"
        $lyxExe = Find-LyX
    }

    Show-InstallProgress -Percent 56 -Status "Detecting Java"
    if (-not $javaExe) {
        Show-InstallProgress -Percent 62 -Status "Installing Java"
        Write-Host "Java not found. Attempting install..."
        [void](Install-Java)
        Show-InstallProgress -Percent 66 -Status "Re-checking Java"
        $javaExe = Find-Java
    }

    Show-InstallProgress -Percent 67 -Status "Detecting LanguageTool"
    $ltDir = Find-LanguageTool
    if (-not $ltDir) {
        [void](Install-LanguageTool)
        Show-InstallProgress -Percent 82 -Status "Re-checking LanguageTool"
        $ltDir = Find-LanguageTool
    }

    return @{
        LyXExe = $lyxExe
        JavaExe = $javaExe
        LanguageTool = $ltDir
    }
}

# Resolve latest release and asset URL for Windows x64
if (-not $AdminPhase) {
    $localChktex = Join-Path $PSScriptRoot "target\release\chktex.exe"
    if (Test-Path $localChktex -PathType Leaf) {
        Write-Host "Using local built binary: $localChktex"
        $dir = Split-Path -Parent $localChktex
        $chktex = Get-Item $localChktex
    } else {
    $api = "https://api.github.com/repos/$REPO/releases/latest"
    Write-Host "Fetching latest release..."
    try {
        $release = Invoke-RestMethod -Uri $api -Headers @{ Accept = "application/vnd.github.v3+json" } -UseBasicParsing
    } catch {
        Write-Host "No release found. Build from source: cd rs && cargo build --release"
        exit 1
    }
    $tag = $release.tag_name
    $asset = $release.assets | Where-Object { $_.name -match "windows.*(x64|x86_64).*\.zip" } | Select-Object -First 1
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
    $chktexExit = $LASTEXITCODE
    Write-Host "Sample run exit code: $chktexExit"
    Write-Host ""
    Write-Host "Demonstrating LyX-style invocation (-x)..." -ForegroundColor Cyan
    & $chktex.FullName -v1 -x $samplePath
    Write-Host "LyX-style run exit code: $LASTEXITCODE"
    Write-Host ""
    Write-Host "Binary: $($chktex.FullName)" -ForegroundColor Green
    Write-Host "Add to PATH or copy to a permanent location."
    Write-Host "Try: chktex yourfile.tex"
}

if (-not $NoLyXGui) {
    Write-Host ""
    Write-Host "Starting full LyX GUI setup..." -ForegroundColor Cyan
    Write-Host "This flow can install LyX, Java, and LanguageTool if missing."
    Write-Host ""
    Show-InstallProgress -Percent 35 -Status "Preparing dependency checks"
    $deps = Ensure-LyXGuiDeps -AllowElevationRelaunch:(-not $AdminPhase)
    Write-Host ""
    Write-Host "Dependency status:"
    Write-Host "  LyX:          $($deps.LyXExe)"
    Write-Host "  Java:         $($deps.JavaExe)"
    Write-Host "  LanguageTool: $($deps.LanguageTool)"
    if (-not $deps.LyXExe) {
        Write-Progress -Activity "LyX GUI setup" -Completed
        Write-Host "LyX still not found. Install LyX and rerun quickstart." -ForegroundColor Red
        exit 1
    }
    Write-Host ""
    Show-InstallProgress -Percent 95 -Status "Launching LyX"
    Write-Host "Launching LyX..." -ForegroundColor Cyan
    if (-not $AdminPhase) {
        if ($chktex -and $chktex.Directory -and (Test-Path $chktex.Directory.FullName)) {
            Write-Host "Injecting chktex directory into LyX PATH: $($chktex.Directory.FullName)"
            $lyxUserDir = Setup-LyXUserDirWithChktex -ChktexExe $chktex.FullName
            if ($lyxUserDir) {
                Write-Host "Using LyX userdir override: $lyxUserDir"
                $proc = New-Object System.Diagnostics.Process
                $proc.StartInfo = New-Object System.Diagnostics.ProcessStartInfo
                $proc.StartInfo.FileName = $deps.LyXExe
                $proc.StartInfo.UseShellExecute = $false
                $proc.StartInfo.WorkingDirectory = Split-Path -Parent $deps.LyXExe
                $proc.StartInfo.Arguments = "-userdir `"$lyxUserDir`""
                $proc.StartInfo.EnvironmentVariables["PATH"] = "$($chktex.Directory.FullName);$($env:PATH)"
                [void]$proc.Start()
            } else {
                Write-Host "Could not create LyX userdir override; using PATH injection only." -ForegroundColor Yellow
                Start-LyXWithChktexPath -LyXExe $deps.LyXExe -ChktexDir $chktex.Directory.FullName
            }
        } else {
            Write-Host "chktex path not available; launching LyX without PATH override." -ForegroundColor Yellow
            Start-Process -FilePath $deps.LyXExe
        }
    } else {
        Write-Host "Admin phase finished dependency installation." -ForegroundColor Green
    }
    Write-Progress -Activity "LyX GUI setup" -Completed
}

if ($AdminPhase) {
    Write-AdminFailureLog
    Write-Host ""
    Write-Host "Admin phase complete. Keeping this window open for 10 seconds..." -ForegroundColor Cyan
    Start-Sleep -Seconds 10
}
