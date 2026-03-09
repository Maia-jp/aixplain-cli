# One-command installer for aix (aiXplain CLI) on Windows
# Usage: irm https://raw.githubusercontent.com/aixplain/aixplain-cli/main/scripts/install.ps1 | iex

$ErrorActionPreference = "Stop"

$Repo = "aixplain/aixplain-cli"
$Binary = "aix.exe"
$InstallDir = if ($env:AIX_INSTALL_DIR) { $env:AIX_INSTALL_DIR } else { "$env:USERPROFILE\.aixplain\bin" }

$Arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { Write-Error "32-bit Windows is not supported"; exit 1 }
$Platform = "${Arch}-pc-windows-msvc"

# Get latest release
$Release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
$Version = $Release.tag_name

$Archive = "aix-${Version}-${Platform}.zip"
$Url = "https://github.com/$Repo/releases/download/$Version/$Archive"

Write-Host "Installing aix $Version for $Platform..."
Write-Host "  Downloading: $Url"

$TmpDir = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }
$ArchivePath = Join-Path $TmpDir $Archive

Invoke-WebRequest -Uri $Url -OutFile $ArchivePath
Expand-Archive -Path $ArchivePath -DestinationPath $TmpDir -Force

# Create install directory if needed
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

Move-Item -Path (Join-Path $TmpDir $Binary) -Destination (Join-Path $InstallDir $Binary) -Force
Remove-Item -Recurse -Force $TmpDir

# Add to PATH if not already there
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    Write-Host "  Added $InstallDir to PATH (restart terminal to take effect)"
}

Write-Host ""
Write-Host "  aix installed to $InstallDir\$Binary" -ForegroundColor Green
Write-Host ""
Write-Host "  Get started:"
Write-Host "    `$env:AIXPLAIN_API_KEY = 'your-key-here'"
Write-Host "    aix models list"
Write-Host "    aix              # launch TUI browser"
Write-Host ""
