#Requires -Version 5.1

param(
    [string]$Prefix = "",
    [switch]$SkipFrontend,
    [switch]$WithDesktop,
    [switch]$Help
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$MinRustVersion = '1.80'
$MinNodeVersion = '20'
$DefaultPrefix = Join-Path $env:LOCALAPPDATA 'hione\bin'
$Binaries = @('hi', 'hi-monitor')

function Write-ColorText {
    param([string]$Text, [string]$Color = 'White')
    try {
        $ConsoleColor = [System.ConsoleColor]$Color
        Write-Host -ForegroundColor $ConsoleColor $Text
    } catch {
        Write-Host $Text
    }
}

function Log-Info {
    param([string]$Message)
    Write-ColorText "[INFO] $Message" 'Green'
}

function Log-Warn {
    param([string]$Message)
    Write-ColorText "[WARN] $Message" 'Yellow'
}

function Log-Error {
    param([string]$Message)
    Write-ColorText "[ERROR] $Message" 'Red'
}

function Log-Step {
    param([string]$Message)
    Write-ColorText "[STEP] $Message" 'Cyan'
}

function Print-Help {
    $HelpText = @"
Hi 安装脚本 (Windows PowerShell)

用法:
    pwsh scripts/install.ps1 [-Options]

选项:
    -Prefix <dir>        自定义安装目录 (默认: $env:LOCALAPPDATA\hione\bin)
    -SkipFrontend        跳过 npm install + npm run build
                         (适用于开发者快速重装)
    -WithDesktop         额外构建 Tauri 桌面应用包
                         (产物在 crates\hi-tauri\src-tauri\target\release\bundle\)
    -Help                打印此帮助信息

示例:
    # 默认安装 (仅 CLI + monitor)
    pwsh scripts/install.ps1

    # 指定安装目录
    pwsh scripts/install.ps1 -Prefix "C:\Tools\hi"

    # 构建桌面应用
    pwsh scripts/install.ps1 -WithDesktop

    # 快速重装 (跳过前端)
    pwsh scripts/install.ps1 -SkipFrontend

前置依赖:
    - Rust >= 1.80       https://rustup.rs
    - Node.js >= 20      https://nodejs.org

安装后请确保安装目录在 PATH 中:
    $env:PATH = "$env:LOCALAPPDATA\hione\bin;$env:PATH"
"@
    Write-Host $HelpText
}

function Check-Command {
    param([string]$Command)
    if (-not (Get-Command $Command -ErrorAction SilentlyContinue)) {
        Log-Error "$Command not found. Please install $Command first."
        switch ($Command) {
            'rustc' { Write-Host '  Install Rust: https://rustup.rs' }
            'cargo' { Write-Host '  Install Rust: https://rustup.rs' }
            'node' { Write-Host '  Install Node.js: https://nodejs.org' }
            'npm' { Write-Host '  Install Node.js: https://nodejs.org' }
        }
        exit 1
    }
}

function Check-Version {
    param([string]$Command, [string]$MinVersion)
    
    switch ($Command) {
        'rustc' {
            $VersionOutput = rustc --version 2>$null
            $ActualVersion = ($VersionOutput -split ' ')[1]
        }
        'cargo' {
            $VersionOutput = cargo --version 2>$null
            $ActualVersion = ($VersionOutput -split ' ')[1]
        }
        'node' {
            $VersionOutput = node --version 2>$null
            $ActualVersion = $VersionOutput.Substring(1)
        }
        default {
            Log-Error "Unknown version check: $Command"
            exit 1
        }
    }
    
    $ActualMajor = $ActualVersion -replace '\..*', ''
    $MinMajor = $MinVersion -replace '\..*', ''
    
    if ([int]$ActualMajor -lt [int]$MinMajor) {
        Log-Error "$Command version $ActualVersion is too old (minimum: $MinVersion)"
        exit 1
    }
    
    Log-Info "$Command version: $ActualVersion (>= $MinVersion OK)"
}

function Check-Dependencies {
    Log-Step 'Checking dependencies...'
    
    Check-Command 'rustc'
    Check-Command 'cargo'
    Check-Command 'node'
    Check-Command 'npm'
    
    Check-Version 'rustc' $MinRustVersion
    Check-Version 'cargo' $MinRustVersion
    Check-Version 'node' $MinNodeVersion
}

function Build-Frontend {
    if ($SkipFrontend) {
        Log-Info 'Skipping frontend build (-SkipFrontend)'
        return
    }
    
    Log-Step 'Building frontend...'
    
    Push-Location 'crates\hi-tauri'
    
    Log-Info 'Running npm install...'
    npm install
    
    Log-Info 'Running npm run build...'
    npm run build
    
    Pop-Location
    
    Log-Info 'Frontend build complete'
}

function Build-Binaries {
    Log-Step 'Building Rust binaries...'
    
    cargo build --workspace --release
    
    Log-Info 'Binaries built'
}

function Build-Desktop {
    if (-not $WithDesktop) {
        return
    }

    Log-Step 'Building Tauri desktop application...'

    cargo tauri build

    Log-Info 'Desktop application built'
    Log-Info 'Bundle location: crates\hi-tauri\src-tauri\target\release\bundle\'
}

function Stop-RunningProcesses {
    $Processes = @('hi', 'hi-monitor', 'hi-tauri')

    foreach ($Proc in $Processes) {
        $running = Get-Process -Name $Proc -ErrorAction SilentlyContinue
        if ($running) {
            Log-Info "Stopping $Proc process..."
            Stop-Process -Name $Proc -Force -ErrorAction SilentlyContinue
            Start-Sleep -Milliseconds 500
        }
    }
}

function Install-Binaries {
    param([string]$InstallPrefix)

    Log-Step "Installing binaries to $InstallPrefix..."

    # Stop running processes to avoid file-in-use errors
    Stop-RunningProcesses

    if (-not (Test-Path $InstallPrefix)) {
        New-Item -ItemType Directory -Path $InstallPrefix -Force | Out-Null
    }

    foreach ($Binary in $Binaries) {
        $Source = "target\release\$Binary.exe"
        $Dest = Join-Path $InstallPrefix "$Binary.exe"

        if (-not (Test-Path $Source)) {
            Log-Error "Binary not found: $Source"
            exit 1
        }

        Copy-Item -Path $Source -Destination $Dest -Force

        Log-Info "Installed: $Dest"
    }

    if ($WithDesktop) {
        $TauriSource = "target\release\hi-tauri.exe"
        if (Test-Path $TauriSource) {
            $TauriDest = Join-Path $InstallPrefix 'hi-tauri.exe'
            Copy-Item -Path $TauriSource -Destination $TauriDest -Force
            Log-Info "Installed: $TauriDest"
        }
    }
}

function Check-Path {
    param([string]$InstallPrefix)
    
    $PathDirs = $env:PATH -split ';'
    $NormalizedPrefix = $InstallPrefix.TrimEnd('\')
    
    $Found = $false
    foreach ($Dir in $PathDirs) {
        if ($Dir.TrimEnd('\') -eq $NormalizedPrefix) {
            $Found = $true
            break
        }
    }
    
    if (-not $Found) {
        Log-Warn "$InstallPrefix is not in your PATH"
        Write-Host ''
        Write-Host '  Add to PATH (PowerShell):'
        Write-Host "    `$env:PATH = '$InstallPrefix;`$env:PATH'"
        Write-Host ''
        Write-Host '  Add to PATH (permanent, user scope):'
        Write-Host "    [Environment]::SetEnvironmentVariable('PATH', '$InstallPrefix;' + [Environment]::GetEnvironmentVariable('PATH', 'User'), 'User')"
    } else {
        Log-Info "$InstallPrefix is in PATH"
    }
}

function Print-WindowsWarning {
    Write-Host ''
    Write-ColorText '======================================== WARNING ======================================' 'Yellow'
    Write-ColorText 'Windows 当前不支持 hi-monitor 的多进程 IPC（Unix Socket 专用）' 'Yellow'
    Write-ColorText '' 'Yellow'
    Write-ColorText '桌面应用可正常运行，但以下命令将返回错误：' 'Yellow'
    Write-ColorText '  - hi push <target> "<content>"' 'Yellow'
    Write-ColorText '  - hi esc <task_id>' 'Yellow'
    Write-ColorText '  - hi pull <target>' 'Yellow'
    Write-ColorText '  - hi check <target>' 'Yellow'
    Write-ColorText '' 'Yellow'
    Write-ColorText '完整跨进程调度请使用 macOS / Linux' 'Yellow'
    Write-ColorText '========================================================================================' 'Yellow'
    Write-Host ''
}

function Print-Success {
    param([string]$InstallPrefix)
    
    Write-Host ''
    Write-ColorText 'Installation complete!' 'Green'
    Write-Host ''
    Write-Host '  Installed binaries:'
    Write-Host "    $InstallPrefix\hi.exe"
    Write-Host "    $InstallPrefix\hi-monitor.exe"
    if ($WithDesktop) {
        Write-Host "    $InstallPrefix\hi-tauri.exe"
    }
    Write-Host ''
    Write-Host '  Quick start:'
    Write-Host '    hi start claude,opencode,gemini'
    Write-Host ''
}

function Main {
    if ($Help) {
        Print-Help
        exit 0
    }
    
    $InstallPrefix = if ($Prefix) { $Prefix } else { $DefaultPrefix }
    
    Write-ColorText 'Hi Installation Script (Windows)' 'Cyan'
    Log-Info "Platform: Windows"
    Log-Info "Install prefix: $InstallPrefix"
    Write-Host ''
    
    Print-WindowsWarning
    
    Check-Dependencies
    
    Build-Frontend
    Build-Binaries
    Build-Desktop
    
    Install-Binaries $InstallPrefix
    
    Check-Path $InstallPrefix
    
    Print-Success $InstallPrefix
}

Main