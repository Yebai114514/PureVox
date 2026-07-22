# 从已生成的 256x256.png 手动构造 ICO 文件
# ICO 格式：6字节header + 16字节entry + 图像数据

$iconDir = Join-Path $PSScriptRoot '..\src-tauri\icons'
$pngPath = Join-Path $iconDir '128x128@2x.png'  # 256x256
$icoPath = Join-Path $iconDir 'icon.ico'

# 读取 PNG 字节
$pngBytes = [System.IO.File]::ReadAllBytes($pngPath)
Write-Host "PNG size: $($pngBytes.Length) bytes"

# 构造 ICO
$ms = New-Object System.IO.MemoryStream
$writer = New-Object System.IO.BinaryWriter $ms

# ICONDIR (6 bytes)
$writer.Write([UInt16]0)             # reserved
$writer.Write([UInt16]1)             # type = icon
$writer.Write([UInt16]1)             # 1 image

# ICONDIRENTRY (16 bytes)
$writer.Write([byte]0)               # width (0 = 256)
$writer.Write([byte]0)               # height (0 = 256)
$writer.Write([byte]0)               # color count
$writer.Write([byte]0)               # reserved
$writer.Write([UInt16]1)             # color planes
$writer.Write([UInt16]32)            # bits per pixel
$writer.Write([UInt32]$pngBytes.Length)  # image size
$writer.Write([UInt32]22)            # offset = 6 + 16 = 22

# 图像数据
$writer.Write($pngBytes)
$writer.Flush()

$icoBytes = $ms.ToArray()
$writer.Close()
$ms.Close()

[System.IO.File]::WriteAllBytes($icoPath, $icoBytes)
Write-Host "Generated ICO: $icoPath ($($icoBytes.Length) bytes)"
