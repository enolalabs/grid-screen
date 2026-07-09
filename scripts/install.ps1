# Grid Screen — Windows Install Script
# Run: irm https://.../install.ps1 | iex

param(
    [string]$InstallDir = "$env:LOCALAPPDATA\grid-screen",
    [string]$BinDir = "$env:LOCALAPPDATA\Programs\grid-screen",
    [string]$InstallMode = "release"
)

$ErrorActionPreference = "Stop"
$RepoUrl = "https://github.com/enolalabs/grid-screen.git"

Write-Host ""
Write-Host "  ╔══════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "  ║       Grid Screen Installer          ║" -ForegroundColor Cyan
Write-Host "  ║   Cross-platform window zone manager ║" -ForegroundColor Cyan
Write-Host "  ╚══════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

function Write-Info  { Write-Host "[INFO] $args" -ForegroundColor Blue }
function Write-Ok    { Write-Host "[OK]   $args" -ForegroundColor Green }
function Write-Warn  { Write-Host "[WARN] $args" -ForegroundColor Yellow }
function Write-Err   { Write-Host "[ERROR] $args" -ForegroundColor Red; exit 1 }

Write-Info "Install dir: $InstallDir"
Write-Info "Binary dir:  $BinDir"

# ── Step 1: Rust ─────────────────────────────────
Write-Host ""
Write-Info "Step 1/6: Checking Rust toolchain..."

if (Get-Command rustc -ErrorAction SilentlyContinue) {
    Write-Ok "Rust already installed: $(rustc --version)"
} else {
    Write-Info "Installing Rust via rustup..."
    Invoke-WebRequest -Uri "https://win.rustup.rs" -OutFile "$env:TEMP\rustup-init.exe"
    & "$env:TEMP\rustup-init.exe" -y --default-toolchain stable
    $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
    Write-Ok "Rust installed"
}

# ── Step 2: Node.js ──────────────────────────────
Write-Host ""
Write-Info "Step 2/6: Checking Node.js..."

if (Get-Command node -ErrorAction SilentlyContinue) {
    Write-Ok "Node.js already installed: $(node --version)"
} else {
    Write-Info "Installing Node.js via winget..."
    winget install OpenJS.NodeJS.LTS --silent --accept-package-agreements
    $env:PATH = "$env:ProgramFiles\nodejs;$env:PATH"
    Write-Ok "Node.js installed"
}

# ── Step 3: Git ──────────────────────────────────
Write-Host ""
Write-Info "Step 3/6: Checking Git..."

if (Get-Command git -ErrorAction SilentlyContinue) {
    Write-Ok "Git already installed"
} else {
    Write-Info "Installing Git via winget..."
    winget install Git.Git --silent --accept-package-agreements
    Write-Ok "Git installed"
}

# ── Step 4: Clone ────────────────────────────────
Write-Host ""
Write-Info "Step 4/6: Cloning repository..."

if (Test-Path "$InstallDir\.git") {
    Write-Info "Repository exists, pulling latest..."
    git -C "$InstallDir" pull --ff-only origin main
    Write-Ok "Repository updated"
} else {
    git clone --depth 1 "$RepoUrl" "$InstallDir"
    Write-Ok "Repository cloned"
}

# ── Step 5: Build ────────────────────────────────
Write-Host ""
Write-Info "Step 5/6: Installing dependencies + building..."

Set-Location $InstallDir
npm install --silent 2>&1 | Select-Object -Last 3

if ($InstallMode -eq "release") {
    cargo build --release 2>&1 | Select-Object -Last 5
    $BinaryPath = "$InstallDir\src-tauri\target\release\grid-screen.exe"
} else {
    cargo build 2>&1 | Select-Object -Last 5
    $BinaryPath = "$InstallDir\src-tauri\target\debug\grid-screen.exe"
}

if (-not (Test-Path $BinaryPath)) {
    Write-Err "Build failed. Check output above."
}
Write-Ok "Build complete"

# ── Step 6: Install ──────────────────────────────
Write-Host ""
Write-Info "Step 6/6: Installing..."

New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
Copy-Item $BinaryPath "$BinDir\grid-screen.exe" -Force

# Add to PATH
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*$BinDir*") {
    [Environment]::SetEnvironmentVariable("PATH", "$userPath;$BinDir", "User")
    Write-Ok "Added to user PATH"
}

# Autostart via registry
$regPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
Set-ItemProperty -Path $regPath -Name "grid-screen" -Value "$BinDir\grid-screen.exe"
Write-Ok "Autostart enabled via registry"

# Start menu shortcut
$shortcutPath = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Grid Screen.lnk"
$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut($shortcutPath)
$Shortcut.TargetPath = "$BinDir\grid-screen.exe"
$Shortcut.Save()
Write-Ok "Start menu shortcut created"

# ── Done ─────────────────────────────────────────
Write-Host ""
Write-Host "  ╔══════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "  ║      Installation Complete!          ║" -ForegroundColor Cyan
Write-Host "  ╚══════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Grid Screen installed. Press Win, type 'grid-screen' to launch."
Write-Host ""
