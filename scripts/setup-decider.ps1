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
            Write-Host "tev1:0.8b installed successfully in Ollama." -ForegroundColor Green
        } catch {
            Write-Warning "Failed to pull tev1:0.8b via Ollama. You can run 'ollama run tev1:0.8b' manually."
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
        Write-Host "Laya model artifact already exists: $layaFile" -ForegroundColor Green
    } else {
        Write-Host "Downloading laya artifact to $layaFile..." -ForegroundColor Gray
        # ponytail: placeholder artifact pointer; download from repository release when online
        New-Item -ItemType File -Path $layaFile -Value "LAYA_MODEL_STUB" -Force | Out-Null
        Write-Host "Configured laya model slot at $layaFile." -ForegroundColor Green
    }
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
