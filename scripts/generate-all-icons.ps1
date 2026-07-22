Add-Type -AssemblyName System.Drawing

$srcPath = Join-Path $PSScriptRoot 'app-icon.png'
$iconDir = Join-Path $PSScriptRoot '..\src-tauri\icons'
if (-not (Test-Path $iconDir)) {
    New-Item -ItemType Directory -Path $iconDir -Force | Out-Null
}

$src = [System.Drawing.Image]::FromFile($srcPath)
Write-Host "Source: $($src.Width)x$($src.Height)"

# 生成不同尺寸 PNG
$pngSizes = @(
    @{ name = '32x32.png';        size = 32 },
    @{ name = '128x128.png';      size = 128 },
    @{ name = '128x128@2x.png';   size = 256 },
    @{ name = 'icon.png';         size = 512 }
)

foreach ($entry in $pngSizes) {
    $outPath = Join-Path $iconDir $entry.name
    $bmp = New-Object System.Drawing.Bitmap($entry.size, $entry.size)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $g.DrawImage($src, 0, 0, $entry.size, $entry.size)
    $g.Dispose()
    $bmp.Save($outPath, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
    Write-Host "Generated PNG: $outPath"
}

# 生成 ICO：使用 Icon.FromHandle（最可靠的方式）
# 先生成 256x256 位图，然后用 Icon 类转换
$bmp256 = New-Object System.Drawing.Bitmap(256, 256)
$g256 = [System.Drawing.Graphics]::FromImage($bmp256)
$g256.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
$g256.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
$g256.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
$g256.DrawImage($src, 0, 0, 256, 256)
$g256.Dispose()

# 用 Icon.FromHandle 生成单尺寸 ICO
$hIcon = $bmp256.GetHicon()
$icon = [System.Drawing.Icon]::FromHandle($hIcon)
$icoPath = Join-Path $iconDir 'icon.ico'
$fs = [System.IO.File]::Create($icoPath)
$icon.Save($fs)
$fs.Close()
[void][System.Runtime.InteropServices.Marshal]::DestroyIcon($hIcon)
$bmp256.Dispose()
Write-Host "Generated ICO: $icoPath"

# 生成 ICNS 占位（macOS 用，Windows 构建不需要但保持配置完整）
# 简单复制 512x512 PNG 作为 icns 占位（实际 icns 格式更复杂，但 Windows 构建不会用到）
$icnsPath = Join-Path $iconDir 'icon.icns'
$bmp512 = New-Object System.Drawing.Bitmap(512, 512)
$g512 = [System.Drawing.Graphics]::FromImage($bmp512)
$g512.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
$g512.DrawImage($src, 0, 0, 512, 512)
$g512.Dispose()
$bmp512.Save($icnsPath, [System.Drawing.Imaging.ImageFormat]::Png)
$bmp512.Dispose()
Write-Host "Generated ICNS (PNG placeholder): $icnsPath"

$src.Dispose()
Write-Host "`nAll icons generated in: $iconDir"
Write-Host "Files:"
Get-ChildItem $iconDir | ForEach-Object { Write-Host "  $($_.Name) - $($_.Length) bytes" }
