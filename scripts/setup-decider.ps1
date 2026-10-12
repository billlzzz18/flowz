param (
    [Parameter(Position = 0)]
    [ValidateSet("tev1", "laya", "all")]
    [string]$Model = "all"
)

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host " Flowz Decider Model Setup (System One)" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

$modelsDir = Join-Path $PSScriptRoot "..\models"
if (-not (Test-Path $modelsDir)) {
    New-Item -ItemType Directory -Path $modelsDir -Force | Out-Null
}

function Setup-Tev1 {
    Write-Host "`n[1/2] Setting up tev1 (0.8B fast model)..." -ForegroundColor Yellow
    $ollama = Get-Command ollama -ErrorAction SilentlyContinue
    if ($ollama) {
        Write-Host "Ollama detected. Pulling tev1:0.8b..." -ForegroundColor Green
        try {
            & ollama pull tev1:0.8b
            if ($LASTEXITCODE -ne 0) {
                throw "ollama pull returned non-zero exit code $LASTEXITCODE"
            }
            Write-Host "tev1:0.8b installed successfully in Ollama." -ForegroundColor Green
        } catch {
            Write-Error "Failed to pull tev1:0.8b via Ollama: $_"
        }
    } else {
        Write-Warning "Ollama is not installed or not in PATH."
        Write-Host "To use tev1 via Ollama, install Ollama from https://ollama.com" -ForegroundColor Gray
    }
}

function Setup-Laya {
    Write-Host "`n[2/2] Setting up laya (jevos GGUF standalone CPU model)..." -ForegroundColor Yellow
    $layaFile = Join-Path $modelsDir "jevos-v2-q4_k_m.gguf"
    if (Test-Path $layaFile) {
        if ((Get-Item $layaFile).Length -lt 1000000) {
            Write-Warning "Existing laya model artifact appears to be an invalid stub (size < 1MB)."
        } else {
            Write-Host "Laya model artifact already exists: $layaFile" -ForegroundColor Green
            return
        }
    }

    Write-Host "Laya standalone model artifact is not bundled in repository." -ForegroundColor Yellow
    Write-Host "Download the real GGUF artifact (jevos-v2-q4_k_m.gguf) into $modelsDir before running standalone mode." -ForegroundColor Yellow
}

switch ($Model) {
    "tev1" { Setup-Tev1 }
    "laya" { Setup-Laya }
    "all" {
        Setup-Tev1
        Setup-Laya
    }
}

Write-Host "`nSetup completed. Models are ready for Flowz Decision Service." -ForegroundColor Cyan
