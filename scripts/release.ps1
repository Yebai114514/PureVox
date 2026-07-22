# PureVox 一键发布脚本
# 用法：在仓库根目录执行  .\scripts\release.ps1
# 前置条件：已安装 pnpm、Rust、GitHub CLI (gh) 且 gh 已登录

$ErrorActionPreference = "Stop"

# 颜色输出函数
function Write-Info($msg) { Write-Host "[INFO] $msg" -ForegroundColor Cyan }
function Write-Success($msg) { Write-Host "[OK] $msg" -ForegroundColor Green }
function Write-Warn($msg) { Write-Host "[WARN] $msg" -ForegroundColor Yellow }
function Write-ErrorLine($msg) { Write-Host "[ERROR] $msg" -ForegroundColor Red }

# 1. 检查 gh 登录状态
Write-Info "检查 GitHub CLI 登录状态..."
$ghStatus = & gh auth status 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-ErrorLine "gh 未登录。请先执行：gh auth login"
    exit 1
}
Write-Success "gh 已登录"

# 2. 读取版本号（优先 Cargo.toml）
$cargoToml = Get-Content "src-tauri\Cargo.toml" -Raw
$version = [regex]::Match($cargoToml, 'version\s*=\s*"([^"]+)"').Groups[1].Value
if (-not $version) {
    Write-ErrorLine "无法从 src-tauri\Cargo.toml 读取版本号"
    exit 1
}
Write-Info "当前版本：$version"

# 3. 确认发布
$confirm = Read-Host "即将创建 GitHub Release v$version 并上传安装包，是否继续？(y/N)"
if ($confirm -ne 'y' -and $confirm -ne 'Y') {
    Write-Warn "已取消"
    exit 0
}

# 4. 检查工作区是否干净
Write-Info "检查 Git 工作区..."
$status = git status --short
if ($status) {
    Write-Warn "工作区有未提交改动："
    Write-Host $status
    $force = Read-Host "是否继续？(y/N)"
    if ($force -ne 'y' -and $force -ne 'Y') {
        Write-Warn "已取消"
        exit 0
    }
}

# 5. 结束正在运行的 PureVox，避免文件锁定
$proc = Get-Process -Name "purevox" -ErrorAction SilentlyContinue
if ($proc) {
    Write-Info "结束正在运行的 purevox.exe..."
    Stop-Process -Name "purevox" -Force
    Start-Sleep -Seconds 1
}

# 6. 构建
Write-Info "开始构建（pnpm tauri build）..."
Push-Location "src-frontend"
try {
    pnpm install
    if ($LASTEXITCODE -ne 0) { throw "pnpm install 失败" }

    pnpm tauri build
    if ($LASTEXITCODE -ne 0) { throw "tauri build 失败" }
} finally {
    Pop-Location
}

# 7. 检查构建产物
$exe = "src-tauri\target\release\purevox.exe"
$msi = "src-tauri\target\release\bundle\msi\PureVox_${version}_x64_en-US.msi"
$nsis = "src-tauri\target\release\bundle\nsis\PureVox_${version}_x64-setup.exe"

$assets = @($exe, $msi, $nsis)
foreach ($asset in $assets) {
    if (-not (Test-Path $asset)) {
        Write-ErrorLine "构建产物不存在：$asset"
        exit 1
    }
}
Write-Success "构建产物检查通过"

# 8. 创建 Release 并上传资源
$tag = "v$version"
Write-Info "创建 GitHub Release $tag ..."

gh release create $tag `
    --title "PureVox $tag" `
    --notes "PureVox $tag 发布。" `
    $exe $msi $nsis

if ($LASTEXITCODE -ne 0) {
    Write-ErrorLine "创建 Release 失败"
    exit 1
}

Write-Success "Release $tag 发布成功！"
Write-Info "访问地址：https://github.com/$(gh repo view --json owner,name -q '.owner.login + "/" + .name')/releases/tag/$tag"
