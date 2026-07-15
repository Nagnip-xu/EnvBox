Add-Type -AssemblyName System.Drawing

$src = Join-Path $PSScriptRoot "..\EnBox.png"
$threshold = 45
$target = 1024

if (-not (Test-Path $src)) {
    throw "Source image not found: $src"
}

$img = [System.Drawing.Image]::FromFile((Resolve-Path $src))
$bmp = New-Object System.Drawing.Bitmap($img.Width, $img.Height, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)

for ($y = 0; $y -lt $img.Height; $y++) {
    for ($x = 0; $x -lt $img.Width; $x++) {
        $c = $img.GetPixel($x, $y)
        if ($c.R -le $threshold -and $c.G -le $threshold -and $c.B -le $threshold) {
            $bmp.SetPixel($x, $y, [System.Drawing.Color]::FromArgb(0, 0, 0, 0))
        } else {
            $bmp.SetPixel($x, $y, [System.Drawing.Color]::FromArgb(255, $c.R, $c.G, $c.B))
        }
    }
}
$img.Dispose()

$minX = $bmp.Width
$minY = $bmp.Height
$maxX = 0
$maxY = 0
for ($y = 0; $y -lt $bmp.Height; $y++) {
    for ($x = 0; $x -lt $bmp.Width; $x++) {
        if ($bmp.GetPixel($x, $y).A -gt 10) {
            if ($x -lt $minX) { $minX = $x }
            if ($y -lt $minY) { $minY = $y }
            if ($x -gt $maxX) { $maxX = $x }
            if ($y -gt $maxY) { $maxY = $y }
        }
    }
}

$w = $maxX - $minX + 1
$h = $maxY - $minY + 1
$cropped = New-Object System.Drawing.Bitmap($w, $h, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
$g = [System.Drawing.Graphics]::FromImage($cropped)
$g.Clear([System.Drawing.Color]::Transparent)
$g.DrawImage($bmp, (New-Object System.Drawing.Rectangle(0, 0, $w, $h)), $minX, $minY, $w, $h, [System.Drawing.GraphicsUnit]::Pixel)
$g.Dispose()
$bmp.Dispose()

$size = [Math]::Max($w, $h)
$square = New-Object System.Drawing.Bitmap($size, $size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
$g2 = [System.Drawing.Graphics]::FromImage($square)
$g2.Clear([System.Drawing.Color]::Transparent)
$g2.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
$g2.DrawImage($cropped, [int](($size - $w) / 2), [int](($size - $h) / 2))
$g2.Dispose()
$cropped.Dispose()

$final = New-Object System.Drawing.Bitmap($target, $target, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
$g3 = [System.Drawing.Graphics]::FromImage($final)
$g3.Clear([System.Drawing.Color]::Transparent)
$g3.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
$g3.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
$g3.DrawImage($square, 0, 0, $target, $target)
$g3.Dispose()
$square.Dispose()

$public = Join-Path $PSScriptRoot "..\public\app-icon.png"
$source = Join-Path $PSScriptRoot "..\src-tauri\icon-source.png"
New-Item -ItemType Directory -Force -Path (Split-Path $public) | Out-Null
$final.Save($public, [System.Drawing.Imaging.ImageFormat]::Png)
$final.Save($source, [System.Drawing.Imaging.ImageFormat]::Png)
$final.Dispose()

Write-Host "OK: crop ${w}x${h} -> ${target}x${target}"
Write-Host "Saved: $public"
Write-Host "Saved: $source"
