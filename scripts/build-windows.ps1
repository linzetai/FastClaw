#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# FastClaw — Windows 本地打包脚本
#
# 用法 (PowerShell):
#   .\scripts\build-windows.ps1              # 正常构建
#   .\scripts\build-windows.ps1 -Release     # 构建 + 生成 latest.json
#   .\scripts\build-windows.ps1 -SkipLint    # 跳过 clippy 检查
#━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

param(
    [switch]$Release,
    [switch]$SkipLint
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$AppDir = Join-Path $ProjectRoot "crates\fastclaw-app"
$TauriDir = Join-Path $AppDir "src-tauri"
$DistDir = Join-Path $ProjectRoot "dist"
$KeyPath = Join-Path $env:USERPROFILE ".tauri\fastclaw.key"

function Log($msg) { Write-Host "▸ $msg" -ForegroundColor Cyan }
function Err($msg) { Write-Host "✗ $msg" -ForegroundColor Red }
function Ok($msg) { Write-Host "✓ $msg" -ForegroundColor Green }

#── 环境检查 ──────────────────────────────────────────────────────────

Log "检查构建环境..."

foreach ($cmd in @("cargo", "pnpm", "node")) {
    if (-not (Get-Command $cmd -ErrorAction SilentlyContinue)) {
        Err "未找到 $cmd，请先安装"
        exit 1
    }
}

if (-not (Test-Path $KeyPath)) {
    Err "签名私钥不存在: $KeyPath"
    Write-Host "  运行以下命令生成:"
    Write-Host "  npx @tauri-apps/cli@latest signer generate --write-keys $KeyPath --force -p `"`""
    exit 1
}

$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content $KeyPath -Raw
if (-not $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD) {
    $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
}

$TauriConf = Get-Content (Join-Path $TauriDir "tauri.conf.json") | ConvertFrom-Json
$Version = $TauriConf.version
Ok "版本号: v$Version"
Ok "Cargo: $(cargo --version)"
Ok "Node:  $(node --version)"
Ok "pnpm:  $(pnpm --version)"

#── Lint ──────────────────────────────────────────────────────────────

if (-not $SkipLint) {
    Log "运行 clippy..."
    Push-Location $ProjectRoot
    cargo clippy --workspace --all-targets -- -D warnings
    if ($LASTEXITCODE -ne 0) { Err "Clippy 检查失败"; exit 1 }
    Pop-Location
    Ok "Clippy 通过"
}

#── 前端构建 ──────────────────────────────────────────────────────────

Log "安装前端依赖..."
Push-Location $AppDir
pnpm install --frozen-lockfile
if ($LASTEXITCODE -ne 0) { Err "pnpm install 失败"; exit 1 }

Log "构建前端..."
pnpm build
if ($LASTEXITCODE -ne 0) { Err "前端构建失败"; exit 1 }
Pop-Location
Ok "前端构建完成"

#── Tauri 构建 ────────────────────────────────────────────────────────

Log "构建 Tauri 应用 (Windows)..."
Push-Location $AppDir
pnpm exec -- tauri build
if ($LASTEXITCODE -ne 0) { Err "Tauri 构建失败"; exit 1 }
Pop-Location
Ok "Tauri 构建完成"

#── 收集产物 ──────────────────────────────────────────────────────────

Log "收集构建产物..."
if (Test-Path $DistDir) { Remove-Item -Recurse -Force $DistDir }
New-Item -ItemType Directory -Force -Path $DistDir | Out-Null

$BundleDir = Join-Path $ProjectRoot "target\release\bundle"
$Patterns = @("*.exe", "*.msi", "*.nsis.zip", "*.nsis.zip.sig")

foreach ($pattern in $Patterns) {
    Get-ChildItem -Recurse -Path $BundleDir -Filter $pattern -ErrorAction SilentlyContinue |
        Copy-Item -Destination $DistDir
}

Ok "产物已收集到 $DistDir\"
Get-ChildItem $DistDir | Format-Table Name, @{Label="Size"; Expression={"{0:N2} MB" -f ($_.Length / 1MB)}} -AutoSize

#── 生成 latest.json (-Release 模式) ─────────────────────────────────

if ($Release) {
    Log "生成 latest.json..."

    $NsisZip = Get-ChildItem $DistDir -Filter "*.nsis.zip" | Where-Object { $_.Name -notmatch "\.sig$" } | Select-Object -First 1
    $NsisSig = Get-ChildItem $DistDir -Filter "*.nsis.zip.sig" | Select-Object -First 1

    if (-not $NsisZip -or -not $NsisSig) {
        Err "未找到 NSIS zip 归档或签名文件"
        exit 1
    }

    $SigContent = Get-Content $NsisSig.FullName -Raw
    $PubDate = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

    $LatestJson = @{
        version = $Version
        notes = "FastClaw v$Version"
        pub_date = $PubDate
        platforms = @{
            "windows-x86_64" = @{
                url = "REPLACE_WITH_DOWNLOAD_URL/$($NsisZip.Name)"
                signature = $SigContent.Trim()
            }
        }
    } | ConvertTo-Json -Depth 4

    $LatestJson | Out-File -FilePath (Join-Path $DistDir "latest.json") -Encoding utf8

    Ok "latest.json 已生成"
    Write-Host ""
    Write-Host "  ⚠ 请编辑 $DistDir\latest.json 中的 url 字段"
    Write-Host "    将 REPLACE_WITH_DOWNLOAD_URL 替换为实际的下载地址"
    Write-Host ""
}

#── 完成 ──────────────────────────────────────────────────────────────

Write-Host ""
Ok "Windows 构建完成! 产物位于: $DistDir\"
Write-Host ""
Write-Host "  产物列表:"
Get-ChildItem $DistDir | ForEach-Object {
    $size = "{0:N2} MB" -f ($_.Length / 1MB)
    Write-Host "    $($_.Name)  ($size)"
}
Write-Host ""

if ($Release) {
    Write-Host "  发布步骤:"
    Write-Host "    1. 上传 dist\ 中的所有文件到发布渠道"
    Write-Host "    2. 编辑 latest.json 中的 url 为实际下载地址"
    Write-Host "    3. 将 latest.json 放到更新端点 URL 可访问的位置"
}
