$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$source = Join-Path $projectRoot "EnvBox.exe"
$destinationDirectory = Join-Path $projectRoot "website\release\downloads"
$destination = Join-Path $destinationDirectory "EnvBox.exe"

if (-not (Test-Path -LiteralPath $source -PathType Leaf)) {
    Write-Output "未找到 $source，已跳过官网安装文件复制。"
    exit 0
}

New-Item -ItemType Directory -Force -Path $destinationDirectory | Out-Null
Copy-Item -LiteralPath $source -Destination $destination -Force
Write-Output "官网安装文件已复制: $destination"
