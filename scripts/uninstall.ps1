#Requires -Version 5.1

param(
    [string]$Prefix = "",
    [switch]$Purge,
    [switch]$Help
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$DefaultPrefix = Join-Path $env:LOCALAPPDATA 'hione\bin'
$Binaries = @('hi', 'hi-monitor', 'hi-tauri')

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
Hi 卸载脚本

用法:
    pwsh scripts/uninstall.ps1 [-Options]

选项:
    -Prefix <dir>    自定义安装目录 (默认: $env:LOCALAPPDATA\hi\bin)
    -Purge           删除 $env:USERPROFILE\.hione 目录 (用户主目录下的全局设置)
    -Help            打印此帮助信息

示例:
    # 默认卸载 (保留 .hione 数据)
    pwsh scripts/uninstall.ps1

    # 完全清理 (删除用户主目录下的 .hione)
    pwsh scripts/uninstall.ps1 -Purge

注意:
    项目目录下的 .hione 不会被删除，这些是项目的会话记录。
"@
    Write-Host $HelpText
}

function Remove-Binaries {
    param([string]$InstallPrefix)
    
    Log-Step "Removing binaries from $InstallPrefix..."
    
    $Removed = 0
    $Skipped = 0
    
    foreach ($Binary in $Binaries) {
        $Path = Join-Path $InstallPrefix "$Binary.exe"
        
        if (Test-Path $Path) {
            Remove-Item -Path $Path -Force
            Log-Info "Removed: $Path"
            $Removed++
        } else {
            Log-Info "Skipped (not found): $Path"
            $Skipped++
        }
    }
    
    Write-Host ''
    Log-Info "Removed: $Removed, Skipped: $Skipped"
}

function Ask-HionePurge {
    if ($Purge) {
        return $true
    }
    
    Write-Host ''
    Write-Host 'Do you want to remove ~/.hione (global settings)? [y/N]'
    
    try {
        $Answer = Read-Host
        switch ($Answer.ToLower()) {
            'y' { $Purge = $true; return $true }
            'yes' { $Purge = $true; return $true }
            default {
                Log-Info 'Keeping .hione data'
                return $false
            }
        }
    } catch {
        Log-Info 'Keeping .hione data (non-interactive mode)'
        return $false
    }
}

function Purge-HioneDirs {
    if (-not $Purge) {
        return
    }
    
    Log-Step 'Removing ~/.hione...'
    
    $HioneDir = Join-Path $env:USERPROFILE '.hione'
    
    if (Test-Path $HioneDir) {
        Remove-Item -Path $HioneDir -Recurse -Force
        Log-Info "Removed: $HioneDir"
    } else {
        Log-Info '~/.hione not found'
    }
    
    Log-Info 'Project .hione directories are preserved (session records)'
}

function Print-Summary {
    Write-Host ''
    Write-ColorText 'Uninstall complete' 'Green'
}

function Main {
    if ($Help) {
        Print-Help
        exit 0
    }
    
    $InstallPrefix = if ($Prefix) { $Prefix } else { $DefaultPrefix }
    
    Write-ColorText 'Hi Uninstall Script (Windows)' 'Cyan'
    Log-Info "Platform: Windows"
    Log-Info "Install prefix: $InstallPrefix"
    Write-Host ''
    
    Remove-Binaries $InstallPrefix
    
    if (Ask-HionePurge) {
        Purge-HioneDirs
    }
    
    Print-Summary
}

Main