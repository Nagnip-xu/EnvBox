$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$package = Get-Content (Join-Path $root "package.json") -Raw -Encoding UTF8 | ConvertFrom-Json
$tauri = Get-Content (Join-Path $root "src-tauri\tauri.conf.json") -Raw -Encoding UTF8 | ConvertFrom-Json
$cargo = Get-Content (Join-Path $root "src-tauri\Cargo.toml") -Raw -Encoding UTF8
$frontend = Get-Content (Join-Path $root "src\version.ts") -Raw -Encoding UTF8

$cargoVersion = [regex]::Match($cargo, '(?m)^version\s*=\s*"([^"]+)"').Groups[1].Value
$frontendVersion = [regex]::Match($frontend, 'APP_VERSION\s*=\s*"([^"]+)"').Groups[1].Value
$versions = @($package.version, $tauri.version, $cargoVersion, $frontendVersion)

if (($versions | Select-Object -Unique).Count -ne 1) {
    throw "版本号不一致：package=$($package.version), tauri=$($tauri.version), cargo=$cargoVersion, frontend=$frontendVersion"
}

Write-Output "Version check passed: $($package.version)"
