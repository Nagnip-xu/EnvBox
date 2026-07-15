# EnvBox Smoke / 全量测试脚本
# 用法：在项目根目录执行  .\scripts\smoke-test.ps1

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

Write-Host "`n=== EnvBox 自动化测试 ===" -ForegroundColor Cyan

Write-Host "`n[1/3] 前端单元测试 (Vitest)..." -ForegroundColor Yellow
npm test
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "`n[2/3] Rust 单元测试..." -ForegroundColor Yellow
Push-Location src-tauri
cargo test --lib
$unitExit = $LASTEXITCODE
Pop-Location
if ($unitExit -ne 0) { exit $unitExit }

Write-Host "`n[3/3] Rust Smoke 测试（只读，需 Windows）..." -ForegroundColor Yellow
Push-Location src-tauri
cargo test --test smoke
$smokeExit = $LASTEXITCODE
Pop-Location
if ($smokeExit -ne 0) { exit $smokeExit }

Write-Host "`n=== 全部通过 ===" -ForegroundColor Green
Write-Host "人工 E2E 请继续对照 TEST_CHECKLIST.md 逐项勾选。" -ForegroundColor Gray
