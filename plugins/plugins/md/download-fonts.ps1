$fonts = @(
    "KaTeX_AMS-Regular.woff2",
    "KaTeX_AMS-Regular.woff",
    "KaTeX_AMS-Regular.ttf",
    "KaTeX_Caligraphic-Bold.woff2",
    "KaTeX_Caligraphic-Bold.woff",
    "KaTeX_Caligraphic-Bold.ttf",
    "KaTeX_Caligraphic-Regular.woff2",
    "KaTeX_Caligraphic-Regular.woff",
    "KaTeX_Caligraphic-Regular.ttf",
    "KaTeX_Fraktur-Bold.woff2",
    "KaTeX_Fraktur-Bold.woff",
    "KaTeX_Fraktur-Bold.ttf",
    "KaTeX_Fraktur-Regular.woff2",
    "KaTeX_Fraktur-Regular.woff",
    "KaTeX_Fraktur-Regular.ttf",
    "KaTeX_Main-Bold.woff2",
    "KaTeX_Main-Bold.woff",
    "KaTeX_Main-Bold.ttf",
    "KaTeX_Main-BoldItalic.woff2",
    "KaTeX_Main-BoldItalic.woff",
    "KaTeX_Main-BoldItalic.ttf",
    "KaTeX_Main-Italic.woff2",
    "KaTeX_Main-Italic.woff",
    "KaTeX_Main-Italic.ttf",
    "KaTeX_Main-Regular.woff2",
    "KaTeX_Main-Regular.woff",
    "KaTeX_Main-Regular.ttf",
    "KaTeX_Math-BoldItalic.woff2",
    "KaTeX_Math-BoldItalic.woff",
    "KaTeX_Math-BoldItalic.ttf",
    "KaTeX_Math-Italic.woff2",
    "KaTeX_Math-Italic.woff",
    "KaTeX_Math-Italic.ttf",
    "KaTeX_SansSerif-Bold.woff2",
    "KaTeX_SansSerif-Bold.woff",
    "KaTeX_SansSerif-Bold.ttf",
    "KaTeX_SansSerif-Italic.woff2",
    "KaTeX_SansSerif-Italic.woff",
    "KaTeX_SansSerif-Italic.ttf",
    "KaTeX_SansSerif-Regular.woff2",
    "KaTeX_SansSerif-Regular.woff",
    "KaTeX_SansSerif-Regular.ttf",
    "KaTeX_Script-Regular.woff2",
    "KaTeX_Script-Regular.woff",
    "KaTeX_Script-Regular.ttf",
    "KaTeX_Size1-Regular.woff2",
    "KaTeX_Size1-Regular.woff",
    "KaTeX_Size1-Regular.ttf",
    "KaTeX_Size2-Regular.woff2",
    "KaTeX_Size2-Regular.woff",
    "KaTeX_Size2-Regular.ttf",
    "KaTeX_Size3-Regular.woff2",
    "KaTeX_Size3-Regular.woff",
    "KaTeX_Size3-Regular.ttf",
    "KaTeX_Size4-Regular.woff2",
    "KaTeX_Size4-Regular.woff",
    "KaTeX_Size4-Regular.ttf",
    "KaTeX_Typewriter-Regular.woff2",
    "KaTeX_Typewriter-Regular.woff",
    "KaTeX_Typewriter-Regular.ttf"
)

$baseUrl = "https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/fonts/"
$fontsDir = Join-Path $PSScriptRoot "fonts"

if (-not (Test-Path $fontsDir)) {
    New-Item -ItemType Directory -Path $fontsDir -Force | Out-Null
}

Write-Host "Start downloading KaTeX font files..." -ForegroundColor Green

$successCount = 0
$failCount = 0

foreach ($font in $fonts) {
    $url = "$baseUrl$font"
    $outputPath = Join-Path $fontsDir $font
    
    try {
        Invoke-WebRequest -Uri $url -OutFile $outputPath -UseBasicParsing
        Write-Host "[OK] $font" -ForegroundColor Green
        $successCount++
    } catch {
        Write-Host "[FAIL] $font - $_" -ForegroundColor Red
        $failCount++
    }
}

Write-Host ""
Write-Host "Download completed!" -ForegroundColor Cyan
Write-Host "Success: $successCount files" -ForegroundColor Green
Write-Host "Failed: $failCount files" -ForegroundColor Red