# Geliştirme Kontrolleri
#
# Kullanım:
#   .\scripts\check.ps1
#   .\scripts\check.ps1 -Headless
#   .\scripts\check.ps1 -SkipTests
#
# -Headless: APTOZA_SKIP_SCREEN_TEST=1 ayarlar ve ekran kaydı testini atlar.
# -SkipTests: Yalnızca biçim/clippy/check çalıştırır.

[CmdletBinding()]
param(
    [switch]$SkipTests,
    [switch]$Headless
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Push-Location $root

function Assert-Command {
    param([string]$Name)
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "'$Name' bulunamadı. Lütfen kurun ve PATH'e ekleyin."
    }
}

try {
    Write-Host "AptozaQ3 geliştirme kontrolleri" -ForegroundColor Cyan
    Write-Host "Klasör: $root"

    Assert-Command "cargo"
    Assert-Command "ffmpeg"
    Assert-Command "ffprobe"

    if (-not (Get-Command "ffplay" -ErrorAction SilentlyContinue)) {
        Write-Warning "'ffplay' bulunamadı. Ses önizlemesi çalışmayabilir."
    }

    Write-Host "`n[1/4] cargo fmt" -ForegroundColor Yellow
    cargo fmt --all -- --check
    if ($LASTEXITCODE -ne 0) { throw "Biçimlendirme hatası. 'cargo fmt --all' çalıştırın." }

    Write-Host "`n[2/4] cargo clippy" -ForegroundColor Yellow
    cargo clippy --all-targets
    if ($LASTEXITCODE -ne 0) { throw "Clippy hata döndürdü." }

    Write-Host "`n[3/4] cargo check" -ForegroundColor Yellow
    cargo check --all-targets
    if ($LASTEXITCODE -ne 0) { throw "cargo check başarısız." }

    if (-not $SkipTests) {
        if ($Headless) {
            $env:APTOZA_SKIP_SCREEN_TEST = "1"
            Write-Host "`nHeadless mod: ekran kaydı testi atlanacak." -ForegroundColor DarkCyan
        }

        Write-Host "`n[4/4] cargo test" -ForegroundColor Yellow
        cargo test --all-targets -- --test-threads=1
        if ($LASTEXITCODE -ne 0) { throw "Testler başarısız." }
    } else {
        Write-Host "`n[4/4] Testler atlandı (-SkipTests)." -ForegroundColor DarkYellow
    }

    Write-Host "`nTüm kontroller başarılı." -ForegroundColor Green
} finally {
    Pop-Location
}
