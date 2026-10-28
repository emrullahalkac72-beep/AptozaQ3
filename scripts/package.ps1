# Windows Release Paketleme
#
# Kullanım:
#   .\scripts\package.ps1
#   .\scripts\package.ps1 -Version 1.0.6
#   .\scripts\package.ps1 -NoBuild
#
# Çıktı:
#   dist/AptozaQ3-<sürüm>-windows-x64.zip
#   dist/SHA256SUMS.txt

[CmdletBinding()]
param(
    [string]$Version = "",
    [switch]$NoBuild
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Push-Location $root

try {
    Write-Host "AptozaQ3 paketleme" -ForegroundColor Cyan
    Write-Host "Klasör: $root"

    if ([string]::IsNullOrWhiteSpace($Version)) {
        $manifest = Get-Content "Cargo.toml" -Raw
        if ($manifest -match '(?m)^version\s*=\s*"([^"]+)"') {
            $Version = $Matches[1]
        } else {
            throw "Sürüm Cargo.toml içinden okunamadı."
        }
    }

    if ($Version -notmatch '^\d+\.\d+\.\d+([+-][0-9A-Za-z.-]+)?$') {
        throw "Geçersiz sürüm: $Version (örn. 1.0.6)"
    }

    Write-Host "Sürüm: $Version"

    if (-not $NoBuild) {
        Write-Host "`nRelease derlemesi..." -ForegroundColor Yellow
        cargo build --release --locked
        if ($LASTEXITCODE -ne 0) { throw "Release derlemesi başarısız." }
    }

    $name = "AptozaQ3-$Version-windows-x64"
    $dist = Join-Path $root "dist"
    $dir = Join-Path $dist $name

    if (Test-Path $dir) {
        Remove-Item $dir -Recurse -Force
    }
    New-Item -ItemType Directory -Force -Path $dir | Out-Null

    $exe = Join-Path $root "target\release\aptoza.exe"
    if (-not (Test-Path $exe)) {
        throw "Derlenmiş EXE bulunamadı: $exe"
    }

    Copy-Item $exe (Join-Path $dir "AptozaQ3.exe")
    Copy-Item "README.md", "LICENSE", "KULLANIM_KILAVUZU.md", `
        "CHANGELOG.md", "CONTRIBUTING.md", "CODE_OF_CONDUCT.md", `
        "SECURITY.md", "SUPPORT.md", "ACKNOWLEDGEMENTS.md" $dir
    Copy-Item "docs" $dir -Recurse

    $zip = Join-Path $dist "${name}.zip"
    if (Test-Path $zip) {
        Remove-Item $zip -Force
    }
    Compress-Archive -Path (Join-Path $dir "*") -DestinationPath $zip -Force

    $hash = (Get-FileHash $zip -Algorithm SHA256).Hash.ToLower()
    "$hash  ${name}.zip" | Set-Content -Path (Join-Path $dist "SHA256SUMS.txt") -Encoding ascii

    Write-Host "`nPaket hazır:" -ForegroundColor Green
    Write-Host "  ZIP: $zip"
    Write-Host "  SHA256: $hash"

    Get-ChildItem $dist | Select-Object Name, Length | Format-Table -AutoSize
} finally {
    Pop-Location
}
