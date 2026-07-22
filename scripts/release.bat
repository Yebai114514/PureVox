@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion

:: PureVox 一键发布脚本（Batch 版本）
:: 用法：在仓库根目录执行 .\scripts\release.bat
:: 前置条件：已安装 pnpm、Rust、GitHub CLI (gh) 且 gh 已登录

echo [INFO] 检查 GitHub CLI 登录状态...
gh auth status >nul 2>&1
if errorlevel 1 (
    echo [ERROR] gh 未登录。请先执行：gh auth login
    exit /b 1
)
echo [OK] gh 已登录

:: 读取版本号（从 src-tauri\Cargo.toml）
echo [INFO] 读取版本号...
set VERSION=
for /f "tokens=2 delims==" %%a in ('findstr /R "^version" src-tauri\Cargo.toml') do (
    set VERSION=%%a
    goto :version_found
)
:version_found
if "!VERSION!"=="" (
    echo [ERROR] 无法从 src-tauri\Cargo.toml 读取版本号
    exit /b 1
)
:: 去除引号和空格
set VERSION=!VERSION:"=!
set VERSION=!VERSION: =!
echo [INFO] 当前版本：!VERSION!

:: 确认发布
set /p CONFIRM="即将创建 GitHub Release v!VERSION! 并上传安装包，是否继续？(y/N) "
if /I not "!CONFIRM!"=="y" (
    echo [WARN] 已取消
    exit /b 0
)

:: 检查工作区是否干净
echo [INFO] 检查 Git 工作区...
for /f "delims=" %%i in ('git status --short') do set DIRTY=%%i
if not "!DIRTY!"=="" (
    echo [WARN] 工作区有未提交改动：
    git status --short
    set /p FORCE="是否继续？(y/N) "
    if /I not "!FORCE!"=="y" (
        echo [WARN] 已取消
        exit /b 0
    )
)

:: 结束正在运行的 PureVox，避免文件锁定
echo [INFO] 检查并结束正在运行的 purevox.exe...
taskkill /F /IM purevox.exe >nul 2>&1
if not errorlevel 1 (
    echo [OK] 已结束 purevox.exe
    timeout /t 1 /nobreak >nul
)

:: 构建
echo [INFO] 开始构建（pnpm tauri build）...
cd src-frontend

call pnpm install
if errorlevel 1 (
    echo [ERROR] pnpm install 失败
    cd ..
    exit /b 1
)

call pnpm tauri build
if errorlevel 1 (
    echo [ERROR] tauri build 失败
    cd ..
    exit /b 1
)

cd ..

:: 检查构建产物
echo [INFO] 检查构建产物...
set EXE=src-tauri\target\release\purevox.exe
set MSI=src-tauri\target\release\bundle\msi\PureVox_!VERSION!_x64_en-US.msi
set NSIS=src-tauri\target\release\bundle\nsis\PureVox_!VERSION!_x64-setup.exe

if not exist "!EXE!" (
    echo [ERROR] 构建产物不存在：!EXE!
    exit /b 1
)
if not exist "!MSI!" (
    echo [ERROR] 构建产物不存在：!MSI!
    exit /b 1
)
if not exist "!NSIS!" (
    echo [ERROR] 构建产物不存在：!NSIS!
    exit /b 1
)
echo [OK] 构建产物检查通过

:: 创建 Release 并上传资源
set TAG=v!VERSION!
echo [INFO] 创建 GitHub Release !TAG! ...
gh release create !TAG! --title "PureVox !TAG!" --notes "PureVox !TAG! 发布。" "!EXE!" "!MSI!" "!NSIS!"
if errorlevel 1 (
    echo [ERROR] 创建 Release 失败
    exit /b 1
)

echo [OK] Release !TAG! 发布成功！
echo [INFO] 访问地址：https://github.com/Yebai114514/PureVox/releases/tag/!TAG!

endlocal
