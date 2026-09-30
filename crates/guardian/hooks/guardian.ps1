# Guardian command hook — PowerShell
# Logs every command before execution.
# Import: . /path/to/guardian.ps1

if (-not $env:GUARDIAN_LOG) { $env:GUARDIAN_LOG = "$env:USERPROFILE\.guardian\command.log" }
if (-not $env:GUARDIAN_ENABLED) { $env:GUARDIAN_ENABLED = "1" }

$dir = Split-Path $env:GUARDIAN_LOG -Parent
if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir -Force | Out-Null }

# PSReadLine history handler — fires on every command
if (Get-Module PSReadLine -ErrorAction SilentlyContinue) {
    $script:_guardian_psreadline = $true
} else {
    $script:_guardian_psreadline = $false
}

function Invoke-GuardianPreExec {
    if ($env:GUARDIAN_ENABLED -ne "1") { return }
    $cmd = $MyInvocation.HistoryEntry.Commandline
    if (-not $cmd) { return }
    $ts = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    "$ts`t$PWD`t$cmd" | Out-File -Append -FilePath $env:GUARDIAN_LOG -Encoding utf8
}

# Hook into prompt — check after each command via $?
# ponytail: PS has no preexec; prompt hook is the closest equivalent
$function:prompt_backup = $function:prompt
$function:prompt = {
    Invoke-GuardianPreExec
    & $function:prompt_backup
}
