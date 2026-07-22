Add-Type -AssemblyName System.Drawing
$size = 1024
$bmp = New-Object System.Drawing.Bitmap($size, $size)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$g.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::AntiAliasGridFit
$g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
$g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality

# 圆角矩形路径
$radius = 220
$path = New-Object System.Drawing.Drawing2D.GraphicsPath
$path.AddArc(0, 0, $radius, $radius, 180, 90)
$path.AddArc($size - $radius, 0, $radius, $radius, 270, 90)
$path.AddArc($size - $radius, $size - $radius, $radius, $radius, 0, 90)
$path.AddArc(0, $size - $radius, $radius, $radius, 90, 90)
$path.CloseFigure()

# 紫蓝渐变填充
$rect = New-Object System.Drawing.Rectangle(0, 0, $size, $size)
$brush = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
    $rect,
    [System.Drawing.Color]::FromArgb(124, 92, 255),
    [System.Drawing.Color]::FromArgb(92, 245, 255),
    [System.Drawing.Drawing2D.LinearGradientMode]::ForwardDiagonal
)
$g.FillPath($brush, $path)

# 内层光晕（半透明白色圆）
$glowPath = New-Object System.Drawing.Drawing2D.GraphicsPath
$glowPath.AddEllipse(280, 220, 460, 460)
$glowBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(60, 255, 255, 255))
$g.FillPath($glowBrush, $glowPath)

# 字母 P（居中绘制）
$font = New-Object System.Drawing.Font('Segoe UI', 580, [System.Drawing.FontStyle]::Bold)
$sf = New-Object System.Drawing.StringFormat
$sf.Alignment = [System.Drawing.StringAlignment]::Center
$sf.LineAlignment = [System.Drawing.StringAlignment]::Center

# 用 float 参数构造 RectangleF
$x = -20.0
$y = -50.0
$w = [float]($size + 40)
$h = [float]($size + 50)
$textRect = New-Object System.Drawing.RectangleF($x, $y, $w, $h)

# 文字阴影
$shadowX = -10.0
$shadowY = -40.0
$shadowRect = New-Object System.Drawing.RectangleF($shadowX, $shadowY, $w, $h)
$shadowBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(80, 0, 0, 0))
$g.DrawString('P', $font, $shadowBrush, $shadowRect, $sf)

# 主文字
$g.DrawString('P', $font, [System.Drawing.Brushes]::White, $textRect, $sf)

# 保存
$outPath = Join-Path $PSScriptRoot 'app-icon.png'
$bmp.Save($outPath, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose()
$bmp.Dispose()
Write-Host "Generated: $outPath"
